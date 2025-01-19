// https://utilityapi.com/docs/greenbutton/xml#:~:text=Green%20Button%20specifies%20a%20full,Atom%20Feed%20or%20downloaded%20individually.

use std::collections::HashMap;

use anyhow::{anyhow, Ok, Result};
use chrono::{DateTime, Datelike, NaiveDateTime, TimeDelta};
use entry::parse_entry;
use local_time_parameters::{
    get_date_from_dst_rule, LocalTimeParameters, LocalTimeParametersSingle,
};
use parse_helpers::enums_to_strings;
use roxmltree::Document;

mod content;
mod entry;
mod gb_type_details;
mod interval_reading;
mod local_time_parameters;
mod parquet_column_writers;
mod parse_helpers;
mod reading_type;
mod time_period;
mod timeseries;

pub use crate::entry::Entries;
pub use crate::interval_reading::IntervalReadings;
pub use crate::reading_type::ReadingTypes;
pub use crate::timeseries::TimeSeries;

pub use gb_type_details::get_gb_type_details;

pub fn denormalize_and_link(
    entries: &Entries,
    interval_readings: &IntervalReadings,
    reading_types: &ReadingTypes,
    local_time_parameters: &LocalTimeParameters,
) -> Result<TimeSeries> {
    // We want to map each entry which contains interval readings to it's reading type.
    // Then we can iterate interval readings and
    // pull out each reading type with a single lookup.

    if local_time_parameters.len() > 1 {
        return Err(anyhow!("Input with multiple LocalTimeParameters is currently unsupported. Please let me know if this would be useful to you!"));
    }

    if local_time_parameters.len() == 0 {
        return Err(anyhow!(
            "Missing LocalTimeParameters. Please let me know if you see this failure."
        ));
    }

    let mut entry_index_by_entry_href = HashMap::<&str, usize>::new();

    for index in 0..entries.len() {
        entry_index_by_entry_href.insert(&entries.href[index], index);
    }

    let mut entry_index_to_reading_type_index: Vec<Option<usize>> = vec![];

    for i in 0..entries.len() {
        let meter_reading_entry_href = entries.related_meter_reading_entry_href[i].as_str();
        if meter_reading_entry_href == "" {
            entry_index_to_reading_type_index.push(None);
            continue;
        }
        let meter_reading_entry_index = *entry_index_by_entry_href
            .get(meter_reading_entry_href)
            .unwrap();

        let reading_type_entry_href =
            &entries.related_reading_type_entry_href[meter_reading_entry_index];

        let reading_type_entry_index = entry_index_by_entry_href[reading_type_entry_href.as_str()];

        let reading_type_entry_entry_type = &entries.entry_type[reading_type_entry_index];

        match reading_type_entry_entry_type {
            entry::EntryType::ReadingTypeWithIndex(reading_type_index) => {
                entry_index_to_reading_type_index.push(Some(*reading_type_index));
            }
            x => return Err(anyhow!("Mismatched reading type {:?}", x)),
        }
    }

    let mut timeseries = TimeSeries::default();
    let quality = enums_to_strings("", "QualityOfReading", &interval_readings.quality);

    let accumulation_behaviour = enums_to_strings(
        "ReadingType",
        "accumulationBehaviour",
        &reading_types.accumulation_behaviour,
    );
    let commodity = enums_to_strings("ReadingType", "commodity", &reading_types.commodity);
    let currency = enums_to_strings("ReadingType", "currency", &reading_types.currency);
    let data_qualifier = enums_to_strings(
        "ReadingType",
        "dataQualifier",
        &reading_types.data_qualifier,
    );
    let flow_direction = enums_to_strings(
        "ReadingType",
        "flowDirection",
        &reading_types.flow_direction,
    );
    let kind = enums_to_strings("ReadingType", "kind", &reading_types.kind);
    let phase = enums_to_strings("ReadingType", "phase", &reading_types.phase);
    let power_of_ten_multiplier = &reading_types.power_of_ten_multiplier;
    let uom = enums_to_strings("ReadingType", "uom", &reading_types.uom);

    let local_time_parameters = LocalTimeParametersSingle {
        dst_start_rule: local_time_parameters.dst_start_rule[0],
        dst_end_rule: local_time_parameters.dst_end_rule[0],
        dst_offset: TimeDelta::seconds(local_time_parameters.dst_offset[0]),
        tz_offset: TimeDelta::seconds(local_time_parameters.tz_offset[0]),
    };

    // Year to start_date, end_date.
    let mut last_year = 0;
    let mut last_dst_start: Option<NaiveDateTime> = None;
    let mut last_dst_end: Option<NaiveDateTime> = None;

    // Now we can map from an interval record href to a reading type index.
    // Memory locality would be better if we did this one column at a time,
    // but because this is somewhat random access anyways, it likely doesn't matter too much.
    for i in 0..interval_readings.len() {
        let entry_index = interval_readings.entry_index[i];
        timeseries.title.push(entries.title[entry_index].clone());

        let ir = &interval_readings;

        timeseries.cost.push(ir.cost[i]);
        timeseries.quality.push(quality[i]);
        timeseries.tou.push(ir.tou[i]);
        timeseries
            .time_period_duration_seconds
            .push(ir.time_period_duration_seconds[i]);

        let unix_timestamp = ir.time_period_start_unix[i];
        let mut date_time = DateTime::from_timestamp(unix_timestamp, 0)
            .unwrap()
            .naive_utc();
        let year = date_time.year();
        if year != last_year {
            last_year = year;
            // TODO: ideally we wouldn't just ignore invalid dst rules, but currently, they're pretty common.
            last_dst_start = get_date_from_dst_rule(local_time_parameters.dst_start_rule, year)
                .unwrap_or_else(|x| {
                    eprintln!("{}", x);
                    None
                });
            last_dst_end = get_date_from_dst_rule(local_time_parameters.dst_end_rule, year)
                .unwrap_or_else(|x| {
                    eprintln!("{}", x);
                    None
                });
        }
        if let (Some(last_dst_start), Some(last_dst_end)) = (last_dst_start, last_dst_end) {
            if last_dst_start < date_time && date_time < last_dst_end {
                date_time += local_time_parameters.dst_offset;
            }
        }
        date_time += local_time_parameters.tz_offset;

        timeseries
            .time_period_start_unix
            .push(ir.time_period_start_unix[i]);

        let rt_index = entry_index_to_reading_type_index[entry_index]
            .ok_or(anyhow!("Missing reading type"))?;

        timeseries.value.push(
            (ir.value[i] as f32) * f32::powf(10 as f32, power_of_ten_multiplier[rt_index] as f32),
        );

        timeseries
            .accumulation_behaviour
            .push(accumulation_behaviour[rt_index]);
        timeseries.commodity.push(commodity[rt_index]);
        timeseries.currency.push(currency[rt_index]);
        timeseries.data_qualifier.push(data_qualifier[rt_index]);
        timeseries.flow_direction.push(flow_direction[rt_index]);
        timeseries.kind.push(kind[rt_index]);
        timeseries.phase.push(phase[rt_index]);
        timeseries.uom.push(uom[rt_index]);
    }

    timeseries.fix_provider_bugs_if_needed(&entries.href[0]);

    return Ok(timeseries);
}

pub fn parse_xml<'a, 'b>(xml: &str) -> Result<TimeSeries> {
    let doc = Document::parse(xml)?;

    let root = doc.root();
    let feed = root
        .children()
        .filter(|x| x.tag_name().name() == "feed")
        .next()
        .ok_or(anyhow!("Missing feed"))?;

    let mut entries = Entries::default();
    let mut interval_readings = IntervalReadings::default();
    let mut reading_types = ReadingTypes::default();
    let mut local_time_parameters = LocalTimeParameters::default();

    for node in feed.children() {
        if node.is_element() && node.tag_name().name() == "entry" {
            let entries_len = entries.len();
            (
                entries,
                interval_readings,
                reading_types,
                local_time_parameters,
            ) = parse_entry(
                entries,
                interval_readings,
                reading_types,
                local_time_parameters,
                node,
                entries_len,
            )?;
        }
    }

    let timeseries = denormalize_and_link(
        &entries,
        &interval_readings,
        &reading_types,
        &local_time_parameters,
    )?;
    return Ok(timeseries);
}
