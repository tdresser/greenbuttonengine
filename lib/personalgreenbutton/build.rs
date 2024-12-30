use anyhow::anyhow;

use itertools::izip;
use polars::df;
use polars::frame::DataFrame;
use polars::lazy::dsl::{col, lit};
use polars::prelude::IntoLazy;
use polars::prelude::{JoinArgs, JoinType, NamedFrom};
use roxmltree::Node;
use std::env;
use std::fs::{read_to_string, File};
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

fn first_child_of_type<'a>(d: Node<'a, 'a>, t: &str) -> Result<Node<'a, 'a>, anyhow::Error> {
    return d
        .children()
        .filter(|x| x.tag_name().name() == t)
        .next()
        .ok_or(anyhow!("Missing {:}", t));
}

fn attr<'a>(d: Node<'a, 'a>, s: &str) -> Result<&'a str, anyhow::Error> {
    return d.attribute(s).ok_or(anyhow!("Missing {:?}", s));
}

fn parse_complex_type(d: Node) -> Result<DataFrame, anyhow::Error> {
    let outer_type_name = d.attribute("name").ok_or(anyhow!("Missing type name"))?;
    let mut field_names: Vec<&str> = vec![];
    let mut field_types: Vec<&str> = vec![];
    match outer_type_name {
        "ReadingType" => {
            for child in d.children() {
                if child.tag_name().name() == "complexContent" {
                    let extension = first_child_of_type(child, "extension")?;
                    //let base = extension.attribute("base").ok_or(anyhow!("Missing base"))?;
                    let sequence = first_child_of_type(extension, "sequence")?;
                    for element in sequence.children() {
                        if element.tag_name().name() != "element" {
                            continue;
                        }
                        field_names.push(attr(element, "name")?);
                        field_types.push(attr(element, "type")?);
                    }
                }
            }
        }
        _ => {}
    }

    return Ok(df!(
        "type" => vec![outer_type_name; field_names.len()],
        "field" => field_names,
        "field_type" => field_types,
    )?);
}

fn parse_simple_enumerated_type(d: Node) -> Result<DataFrame, anyhow::Error> {
    let outer_type_name = d.attribute("name").ok_or(anyhow!("Missing type name"))?;
    let mut values: Vec<Option<i32>> = vec![];
    let mut app_infos: Vec<Option<&str>> = vec![];
    let mut documentations: Vec<&str> = vec![];

    let union = first_child_of_type(d, "union");
    if union.is_err() {
        return df!(
            "field_type" => Vec::<&str>::new(),
            "value" => values,
            "app_info" => app_infos,
            "documentation" => documentations
        )
        .map_err(|_| anyhow!("can't construct empty Dataframe."));
    }
    let simple_type = first_child_of_type(union?, "simpleType")?;
    let restriction = first_child_of_type(simple_type, "restriction")?;

    for enumeration in restriction.children() {
        if enumeration.tag_name().name() != "enumeration" {
            continue;
        }
        let annotation = first_child_of_type(enumeration, "annotation")?;
        let documentation = first_child_of_type(annotation, "documentation")?;
        documentations.push(
            documentation
                .text()
                .ok_or(anyhow!("Missing documentation text"))?,
        );

        let app_info = first_child_of_type(annotation, "appinfo");
        values.push(Some(attr(enumeration, "value")?.parse()?));

        if app_info.is_err() {
            app_infos.push(None);
            continue;
        }
        app_infos.push(Some(
            app_info?.text().ok_or(anyhow!("Missing appInfo text"))?,
        ));
    }

    return df!(
        "field_type" => vec![outer_type_name; app_infos.len()],
        "value" => values,
        "app_info" => app_infos,
        "documentation" => documentations
    )
    .map_err(|_| anyhow!("can't construct empty Dataframe."));
}

fn parse(s: &str) -> Result<(), anyhow::Error> {
    let doc = roxmltree::Document::parse(&s)?;
    let mut simple_types_df = DataFrame::empty();
    let mut complex_types_df = DataFrame::empty();
    // First child is Schema.
    for d in doc
        .root()
        .first_child()
        .ok_or(anyhow!("Missing first child."))?
        .children()
    {
        match d.tag_name().name() {
            "complexType" => {
                complex_types_df = complex_types_df.vstack(&parse_complex_type(d)?)?;
            }
            "simpleType" => {
                simple_types_df = simple_types_df.vstack(&parse_simple_enumerated_type(d)?)?;
            }
            _ => {}
        }
    }

    let mut joined = complex_types_df
        .lazy()
        .join(
            simple_types_df.clone().lazy(),
            [col("field_type")],
            [col("field_type")],
            JoinArgs::new(JoinType::Left),
        )
        // We don't need field_type
        .select(&[
            col("type"),
            col("field"),
            col("value"),
            col("app_info"),
            col("documentation"),
        ])
        .collect()?;

    let simple_types_unified = simple_types_df
        .lazy()
        .select(&[
            lit("").alias("type"),
            col("field_type").alias("field"),
            col("value"),
            col("app_info"),
            col("documentation"),
        ])
        .collect()?;

    joined = joined
        .vstack(&simple_types_unified)?
        .lazy()
        // If we don't get rid of these nulls, writing explodes with an internal error.
        .filter(col("value").is_not_null())
        .filter(
            col("type")
                .eq(lit("ReadingType"))
                .or(col("field").eq(lit("QualityOfReading"))),
        )
        .collect()?;

    let mut app_info_map: phf_codegen::Map<String> = phf_codegen::Map::new();

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("espixsd.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    fn string_col<'a>(df: &'a DataFrame, col: &'a str) -> Vec<String> {
        return df
            .column(col)
            .unwrap()
            .str()
            .unwrap()
            .into_iter()
            .map(|x| x.unwrap().replace("\"", "\\\""))
            .collect();
    }

    fn i32_col<'a>(df: &'a DataFrame, col: &'a str) -> Vec<Option<i32>> {
        return df.column(col).unwrap().i32().unwrap().into_iter().collect();
    }

    let xml_type = string_col(&joined, "type");
    let field = string_col(&joined, "field");
    let value = i32_col(&joined, "value");
    let app_info = string_col(&joined, "app_info");
    let documentation = string_col(&joined, "documentation");

    for (xml_type, field, value, app_info, documentation) in
        izip!(xml_type, field, value, app_info, documentation)
    {
        app_info_map.entry(
            format!("{}œ{}œ{}", xml_type, field, value.unwrap()),
            &format!("(\"{}\", \"{}\")", &app_info, &documentation),
        );
    }

    // xml_type œ field œ value => (app_info, documentation)
    write!(
        &mut file,
        "static GB_TYPE_DETAILS: phf::Map<&str, (&'static str, &'static str)> = {}",
        app_info_map.build()
    )
    .unwrap();
    write!(&mut file, ";\n").unwrap();
    return Ok(());
}

fn main() {
    let input_path = "preprocessing/espi.xsd.xml";
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed={}", input_path);

    let str = read_to_string(input_path).unwrap();
    parse(&str).unwrap();
}
