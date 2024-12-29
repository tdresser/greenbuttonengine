use roxmltree::Node;
use std::str::FromStr;

use crate::get_gb_type_details;

pub fn strip_espi_prefix<'a>(x: &'a str) -> &'a str {
    const ESPI_PREFIX_LEN: usize = "{http://naesb.org/espi}".len();
    if x.len() < ESPI_PREFIX_LEN {
        return x;
    }
    return x[ESPI_PREFIX_LEN..].as_ref();
}

pub fn all_text<'a>(node: Node<'a, '_>) -> String {
    return node
        .descendants()
        .map(|x| {
            if x.is_text() {
                return x.text().unwrap_or("").trim();
            }
            return "";
        })
        .collect::<Vec<&str>>()
        .join("");
}

pub fn parse_text_of<'a, T: FromStr + Default>(node: Node<'a, '_>) -> Result<T, anyhow::Error>
where
    <T as FromStr>::Err: std::error::Error,
    <T as FromStr>::Err: Send,
    <T as FromStr>::Err: Sync,
    <T as FromStr>::Err: 'static,
{
    let all_text = all_text(node);
    // This shouldn't happen based on the spec, but Hydro One sometimes has empty cost tags.
    if all_text.is_empty() {
        return Ok(T::default());
    }
    return Ok(all_text.parse::<T>()?);
}

pub fn enums_to_strings<'a>(
    scope: &str, // XML tag scope.
    field: &str, // Which field to map.
    values: &[i32],
) -> Vec<&'a str> {
    return (0..values.len())
        .map(|i| get_gb_type_details(scope, field, values[i]).app_info)
        .collect();
}
