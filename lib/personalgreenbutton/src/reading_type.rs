use anyhow::Result;
use columnar_struct_vec::columnar_struct_vec;
use roxmltree::Node;

use crate::parse_helpers::{parse_text_of, strip_espi_prefix};

#[derive(Debug)]
#[columnar_struct_vec]
pub struct ReadingTypes {
    pub entry_index: usize,
    pub accumulation_behaviour: i32,
    pub commodity: i32,
    pub currency: i32,
    pub data_qualifier: i32,
    pub flow_direction: i32,
    pub kind: i32,
    pub power_of_ten_multiplier: i32,
    // Use "none" if this is missing.
    #[struct_builder(default = 0)]
    pub phase: i32,
    pub uom: i32,
}

pub fn parse_reading_types<'a>(
    reading_types: ReadingTypes,
    node: Node<'a, '_>,
    entry_index: usize,
) -> Result<ReadingTypes> {
    let mut builder = reading_types.start_push();
    builder.entry_index(entry_index);

    for child in node.children() {
        match strip_espi_prefix(child.tag_name().name()) {
            "accumulationBehaviour" => {
                builder.accumulation_behaviour(parse_text_of(child)?);
            }
            "commodity" => {
                builder.commodity(parse_text_of(child)?);
            }
            "currency" => {
                builder.currency(parse_text_of(child)?);
            }
            "dataQualifier" => {
                builder.data_qualifier(parse_text_of(child)?);
            }
            "flowDirection" => {
                builder.flow_direction(parse_text_of(child)?);
            }
            "kind" => builder.kind(parse_text_of(child)?),
            "powerOfTenMultiplier" => {
                builder.power_of_ten_multiplier(parse_text_of(child)?);
            }
            "phase" => {
                builder.phase(parse_text_of(child)?);
            }
            "uom" => {
                builder.uom(parse_text_of(child)?);
            }
            _ => (),
        }
    }
    return builder.finalize_push();
}
