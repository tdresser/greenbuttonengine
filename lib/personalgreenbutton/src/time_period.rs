use anyhow::Result;
use roxmltree::Node;

use crate::{
    interval_reading::IntervalReadingsRowBuilder,
    parse_helpers::{parse_text_of, strip_espi_prefix},
};
use anyhow::anyhow;

pub fn parse_time_period_data(
    mut row_builder: IntervalReadingsRowBuilder,
    node: Node,
) -> Result<IntervalReadingsRowBuilder> {
    let mut duration_child: Option<Node> = None;
    let mut start_child: Option<Node> = None;
    for child in node.children() {
        match strip_espi_prefix(child.tag_name().name()) {
            "start" => start_child = Some(child),
            "duration" => duration_child = Some(child),
            _ => (),
        }
    }
    let seconds: i64 = parse_text_of(start_child.ok_or(anyhow!("Missing start time."))?)?;
    row_builder.time_period_start_unix(seconds);
    row_builder.time_period_duration_seconds(parse_text_of(
        duration_child.ok_or(anyhow!("Missing duration"))?,
    )?);
    return Ok(row_builder);
}
