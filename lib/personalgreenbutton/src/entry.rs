use chrono::DateTime;
use once_cell::sync::Lazy;
use regex::Regex;
use roxmltree::Node;

use anyhow::anyhow;
use anyhow::Result;

use columnar_struct_vec::columnar_struct_vec;

use crate::content::parse_content_data;
use crate::interval_reading::IntervalReadings;
use crate::local_time_parameters::LocalTimeParameters;
use crate::reading_type::ReadingTypes;

#[derive(Debug)]
#[columnar_struct_vec]
pub struct Entries {
    pub entry_type: EntryType,
    pub href: String,
    pub title: String,
    // RFC 3339.
    pub published_unix_ms: i64,
    pub updated_unix_ms: i64,
    // If a meter reading exists.
    #[struct_builder(default)]
    pub related_meter_reading_entry_href: String,
    // If a reading type exists.
    #[struct_builder(default)]
    pub related_reading_type_entry_href: String,
}

trait AttrEquals {
    fn attr_equals(&self, attr: &str, val: &str) -> bool;
}

impl AttrEquals for Node<'_, '_> {
    fn attr_equals(&self, attr: &str, val: &str) -> bool {
        if let Some(attr_val) = self.attribute(attr) {
            return attr_val == val;
        }
        return false;
    }
}

fn get_meter_reading<'a>(href: &'a str) -> Option<&'a str> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new("(.*MeterReading/[^/]*)/").unwrap());

    let captures = RE.captures(&href);
    match captures {
        Some(caps) => {
            if caps.len() > 1 {
                let meter_reading = caps.get(1).unwrap().as_str();
                Some(meter_reading)
            } else {
                None
            }
        }
        None => None,
    }
}

fn parse_link(entries_row_builder: &mut EntriesRowBuilder, node: Node) -> Result<()> {
    if let Some(href) = node.attribute("href") {
        if node.attr_equals("rel", "related") {
            if node.attr_equals("type", "espi-entry/ReadingType") {
                entries_row_builder.related_reading_type_entry_href(href.to_string());
            }
        }
        if node.attr_equals("rel", "self") {
            entries_row_builder.href(href.to_string());
            if let Some(href) = get_meter_reading(href) {
                entries_row_builder.related_meter_reading_entry_href(href.to_string());
            }
        }
    }
    return Ok(());
}

pub fn parse_entry(
    entries: Entries,
    interval_readings: IntervalReadings,
    reading_types: ReadingTypes,
    local_time_parameters: LocalTimeParameters,
    node: Node,
    index: usize,
) -> Result<(Entries, IntervalReadings, ReadingTypes, LocalTimeParameters)> {
    let mut row_builder = entries.start_push();
    let children = node.children();

    let mut content_node: Option<Node> = None;
    for child in children {
        match child.tag_name().name() {
            "title" => row_builder.title(child.text().ok_or(anyhow!("Empty title."))?.to_string()),
            "published" => {
                let text = child.text().ok_or(anyhow!("Missing published text"))?;
                row_builder.published_unix_ms(
                    DateTime::parse_from_rfc3339(text)?
                        .naive_local()
                        .and_utc()
                        .timestamp(),
                );
            }
            "updated" => {
                let text = child.text().ok_or(anyhow!("Missing updated text"))?;
                row_builder.updated_unix_ms(
                    DateTime::parse_from_rfc3339(text)?
                        .naive_local()
                        .and_utc()
                        .timestamp(),
                );
            }
            "content" => content_node = Some(child),
            "link" => {
                parse_link(&mut row_builder, child)?;
            }
            _ => (),
        }
    }

    let (entry_type, interval_readings, reading_types, local_time_parameters) = parse_content_data(
        index,
        interval_readings,
        reading_types,
        local_time_parameters,
        content_node.ok_or(anyhow!("Missing content node"))?,
    )?;

    row_builder.entry_type(entry_type);
    return Ok((
        row_builder.finalize_push()?,
        interval_readings,
        reading_types,
        local_time_parameters,
    ));
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EntryType {
    ReadingTypeWithIndex(usize),
    IntervalBlock,
    LocalTimeParameters,
    Other,
    Unset,
}

impl std::fmt::Display for EntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            EntryType::ReadingTypeWithIndex(_) => "reading type",
            EntryType::IntervalBlock => "interval block",
            EntryType::LocalTimeParameters => "local time parameters",
            EntryType::Other => "unparsed",
            EntryType::Unset => "ERROR",
        };
        return write!(f, "{}", s);
    }
}

impl EntryType {
    pub fn set(&mut self, new: EntryType) -> Result<()> {
        if *self == new {
            return Ok(());
        }
        if *self == EntryType::Unset {
            *self = new;
            return Ok(());
        }
        return Err(anyhow!("Entry has mixed content types."));
    }
}
