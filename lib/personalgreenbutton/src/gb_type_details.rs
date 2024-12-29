include!(concat!(env!("OUT_DIR"), "/espixsd.rs"));

pub struct GreenButtonFieldMetadata<'a> {
    pub app_info: &'a str,
    pub description: &'a str,
}

pub fn get_gb_type_details<'a>(
    xml_type: &str,
    field: &str,
    value: i32,
) -> GreenButtonFieldMetadata<'a> {
    // TODO: figure out a less terrible way to get this map working.
    // We run into awkward lifetime issues if we try to pass in a struct holding &str's.
    let result = GB_TYPE_DETAILS.get(format!("{}œ{}œ{}", xml_type, field, value).as_str());

    match result {
        Some(x) => {
            return GreenButtonFieldMetadata {
                app_info: x.0,
                description: x.1,
            }
        }
        None => {
            return GreenButtonFieldMetadata {
                app_info: "Missing app info",
                description: "Missing description",
            }
        }
    }
}
