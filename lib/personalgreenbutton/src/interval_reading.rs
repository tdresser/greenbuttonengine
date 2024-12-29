use anyhow::anyhow;
use anyhow::Result;
use columnar_struct_vec::columnar_struct_vec;
use roxmltree::Node;

use crate::{
    parse_helpers::{parse_text_of, strip_espi_prefix},
    time_period::parse_time_period_data,
};

#[derive(Debug)]
#[columnar_struct_vec]
pub struct IntervalReadings {
    pub entry_index: usize,
    #[struct_builder(default = "NaN")]
    pub cost: f32,
    // Default to "other".
    #[struct_builder(default = 16)]
    pub quality: i32,
    pub value: i64,
    #[struct_builder(default = 0)]
    pub tou: i32,
    pub time_period_start_unix_ms: i64,
    pub time_period_duration_seconds: i32,
}

fn parse_interval_reading<'a>(
    ir: IntervalReadings,
    node: Node<'a, '_>,
    entry_index: usize,
) -> Result<IntervalReadings> {
    let mut ir = ir.start_push();
    ir.entry_index(entry_index);
    for child in node.children() {
        match strip_espi_prefix(child.tag_name().name()) {
            // Convert to dollars.
            // See https://utilityapi.com/docs/greenbutton/xml#IntervalBlock.
            "cost" => ir.cost(parse_text_of::<f32>(child)? / 100000.0),
            "ReadingQuality" => ir.quality(parse_text_of(child)?),
            "value" => ir.value(parse_text_of(child)?),
            "tou" => ir.tou(parse_text_of(child)?),
            "timePeriod" => ir = parse_time_period_data(ir, child)?,
            _ => {
                if child.tag_name().name().len() > 0 {
                    return Err(anyhow!("Unmatched tag name: {:?}", child.tag_name().name()));
                }
            }
        }
    }

    return Ok(ir.finalize_push()?);
}

pub fn parse_interval_readings(
    mut interval_readings: IntervalReadings,
    node: Node,
    entry_index: usize,
) -> Result<IntervalReadings> {
    for child in node.children() {
        match strip_espi_prefix(child.tag_name().name()) {
            "IntervalReading" => {
                interval_readings = parse_interval_reading(interval_readings, child, entry_index)?;
            }
            _ => (),
        }
    }
    return Ok(interval_readings);
}
