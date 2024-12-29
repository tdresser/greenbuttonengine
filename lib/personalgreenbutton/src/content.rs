use roxmltree::Node;

use anyhow::anyhow;
use anyhow::Result;

use crate::{
    entry::EntryType,
    interval_reading::{parse_interval_readings, IntervalReadings},
    reading_type::{parse_reading_types, ReadingTypes},
};

pub fn parse_content_data(
    entry_index: usize,
    mut interval_readings: IntervalReadings,
    mut reading_types: ReadingTypes,
    node: Node,
) -> Result<(EntryType, IntervalReadings, ReadingTypes)> {
    let mut entry_type = EntryType::Unset;
    // There should only be one interval block node, but some providers (e.g., Hydro One) include
    // multiple in a single content block. I've provided them feedback, but until this is fixed, we'll
    // just parse it anyways.
    let mut interval_block_nodes: Vec<Node> = vec![];
    let mut reading_type_node: Option<Node> = None;
    let children = node.children().filter(|x| x.is_element());
    for child in children {
        match child.tag_name().name() {
            "IntervalBlock" => {
                entry_type.set(EntryType::IntervalBlock)?;
                interval_block_nodes.push(child);
            }
            "ElectricPowerQualitySummary" => entry_type.set(EntryType::Other)?,
            "LocalTimeParameters" => entry_type.set(EntryType::Other)?,
            "MeterReading" => entry_type.set(EntryType::Other)?,
            "ReadingType" => {
                entry_type.set(EntryType::ReadingTypeWithIndex(reading_types.len()))?;
                reading_type_node = Some(child);
            }
            "UsagePoint" => entry_type.set(EntryType::Other)?,
            "UsageSummary" => entry_type.set(EntryType::Other)?,
            _ => return Err(anyhow!("Unknown tag name {:?}", child.tag_name().name())),
        }
    }
    for interval_block_node in interval_block_nodes {
        interval_readings =
            parse_interval_readings(interval_readings, interval_block_node, entry_index)?;
    }
    if let Some(reading_type_node) = reading_type_node {
        reading_types = parse_reading_types(reading_types, reading_type_node, entry_index)?;
    }
    return Ok((entry_type, interval_readings, reading_types));
}
