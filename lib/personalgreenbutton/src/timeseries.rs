use parquet::{
    file::{
        properties::WriterProperties,
        writer::{SerializedFileWriter, TrackedWrite},
    },
    schema::parser::parse_message_type,
};
use regex::Regex;
use std::sync::Arc;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::parquet_column_writers::{write_f32s, write_i32s, write_i64s, write_strs};

// The initial version of this was mostly generated via procedural macro,
// but the complexity didn't seem worth it. There's a lot of boilerplate
// here, but it's probably the best approach regardless.

// All strings are pulled from the static gb_type_details.

#[wasm_bindgen]
#[derive(Debug, Default, Clone)]
pub struct TimeSeries {
    // Entry.
    #[wasm_bindgen(skip)]
    pub title: Vec<String>,

    // Interval Reading.
    #[wasm_bindgen(skip)]
    pub cost: Vec<f32>,
    #[wasm_bindgen(skip)]
    pub quality: Vec<&'static str>,
    #[wasm_bindgen(skip)]
    pub value: Vec<f32>,
    #[wasm_bindgen(skip)]
    pub tou: Vec<i32>,
    #[wasm_bindgen(skip)]
    pub time_period_start_unix_ms: Vec<i64>,
    #[wasm_bindgen(skip)]
    pub time_period_duration_seconds: Vec<i32>,

    // Reading type.
    #[wasm_bindgen(skip)]
    pub accumulation_behaviour: Vec<&'static str>,
    #[wasm_bindgen(skip)]
    pub commodity: Vec<&'static str>,
    #[wasm_bindgen(skip)]
    pub currency: Vec<&'static str>,
    #[wasm_bindgen(skip)]
    pub data_qualifier: Vec<&'static str>,
    #[wasm_bindgen(skip)]
    pub flow_direction: Vec<&'static str>,
    #[wasm_bindgen(skip)]
    pub kind: Vec<&'static str>,
    #[wasm_bindgen(skip)]
    pub phase: Vec<&'static str>,
    #[wasm_bindgen(skip)]
    pub uom: Vec<&'static str>,
}

impl TimeSeries {
    // Requires sorted data.
    pub fn take_first_title_chunk(&mut self) -> Option<TimeSeries> {
        if self.value.is_empty() {
            return None;
        }
        let first_title = &self.title[0];
        let mut after_first_chunk_index: Option<usize> = None;
        for (index, title) in self.title.iter().enumerate() {
            if title != first_title {
                after_first_chunk_index = Some(index);
                break;
            }
        }
        let after_first_chunk_index = if let Some(after_first_chunk_index) = after_first_chunk_index
        {
            after_first_chunk_index
        } else {
            self.value.len()
        };
        let first_chunk = TimeSeries {
            title: self.title.drain(0..after_first_chunk_index).collect(),
            cost: self.cost.drain(0..after_first_chunk_index).collect(),
            quality: self.quality.drain(0..after_first_chunk_index).collect(),
            value: self.value.drain(0..after_first_chunk_index).collect(),
            tou: self.tou.drain(0..after_first_chunk_index).collect(),
            time_period_start_unix_ms: self
                .time_period_start_unix_ms
                .drain(0..after_first_chunk_index)
                .collect(),
            time_period_duration_seconds: self
                .time_period_duration_seconds
                .drain(0..after_first_chunk_index)
                .collect(),
            accumulation_behaviour: self
                .accumulation_behaviour
                .drain(0..after_first_chunk_index)
                .collect(),
            commodity: self.commodity.drain(0..after_first_chunk_index).collect(),
            currency: self.currency.drain(0..after_first_chunk_index).collect(),
            data_qualifier: self
                .data_qualifier
                .drain(0..after_first_chunk_index)
                .collect(),
            flow_direction: self
                .flow_direction
                .drain(0..after_first_chunk_index)
                .collect(),
            kind: self.kind.drain(0..after_first_chunk_index).collect(),
            phase: self.phase.drain(0..after_first_chunk_index).collect(),
            uom: self.uom.drain(0..after_first_chunk_index).collect(),
        };

        return Some(first_chunk);
    }

    pub fn sort(&mut self) {
        let indices: Vec<_> = (0..self.value.len()).collect();
        let mut p = permutation::sort_unstable_by(indices, |i, j| {
            self.title[*i]
                .cmp(&self.title[*j])
                .then(self.time_period_start_unix_ms[*i].cmp(&self.time_period_start_unix_ms[*j]))
        });
        p.apply_slice_in_place(&mut self.title);
        p.apply_slice_in_place(&mut self.cost);
        p.apply_slice_in_place(&mut self.quality);
        p.apply_slice_in_place(&mut self.value);
        p.apply_slice_in_place(&mut self.tou);
        p.apply_slice_in_place(&mut self.time_period_start_unix_ms);
        p.apply_slice_in_place(&mut self.time_period_duration_seconds);
        p.apply_slice_in_place(&mut self.accumulation_behaviour);
        p.apply_slice_in_place(&mut self.commodity);
        p.apply_slice_in_place(&mut self.currency);
        p.apply_slice_in_place(&mut self.data_qualifier);
        p.apply_slice_in_place(&mut self.flow_direction);
        p.apply_slice_in_place(&mut self.kind);
        p.apply_slice_in_place(&mut self.phase);
        p.apply_slice_in_place(&mut self.uom);
    }

    pub fn sort_and_chunk(mut self) -> Vec<TimeSeries> {
        self.sort();
        let mut chunks: Vec<TimeSeries> = vec![];
        while let Some(chunk) = self.take_first_title_chunk() {
            chunks.push(chunk);
        }
        return chunks;
    }

    pub fn extend(&mut self, other: TimeSeries) {
        self.title.extend(other.title);

        // Interval Reading.
        self.cost.extend(other.cost);
        self.quality.extend(other.quality);
        self.value.extend(other.value);
        self.tou.extend(other.tou);
        self.time_period_start_unix_ms
            .extend(other.time_period_start_unix_ms);
        self.time_period_duration_seconds
            .extend(other.time_period_duration_seconds);
        // Reading type.
        self.accumulation_behaviour
            .extend(other.accumulation_behaviour);
        self.commodity.extend(other.commodity);
        self.currency.extend(other.currency);
        self.data_qualifier.extend(other.data_qualifier);
        self.flow_direction.extend(other.flow_direction);
        self.kind.extend(other.kind);
        self.phase.extend(other.phase);
        self.uom.extend(other.uom);
    }

    pub fn fix_provider_bugs_if_needed(&mut self, href: &str) {
        if href.contains("enova") {
            self.cost = self.cost.iter().map(|cost| cost * 100.0).collect();
        }
    }
}

#[wasm_bindgen]
impl TimeSeries {
    #[wasm_bindgen(js_name = "hasCost")]
    pub fn has_cost(&self) -> bool {
        for cost in &self.cost {
            if cost.is_finite() && *cost != 0.0 {
                return true;
            }
        }
        return false;
    }

    #[wasm_bindgen(js_name = "asCSV")]
    pub fn as_csv(&self) -> Result<String, String> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.write_record(&[
            "title",
            "cost",
            "quality",
            "value",
            "tou",
            "time_period_start_unix_ms",
            "time_period_duration_seconds",
            "accumulation_behaviour",
            "commodity",
            "currency",
            "data_qualifier",
            "flow_direction",
            "kind",
            "phase",
            "uom",
        ])
        .map_err(|x| x.to_string())?;

        for i in 0..self.title.len() {
            wtr.write_record(&[
                self.title[i].to_string(),
                self.cost[i].to_string(),
                self.quality[i].to_string(),
                self.value[i].to_string(),
                self.tou[i].to_string(),
                self.time_period_start_unix_ms[i].to_string(),
                self.time_period_duration_seconds[i].to_string(),
                self.accumulation_behaviour[i].to_string(),
                self.commodity[i].to_string(),
                self.currency[i].to_string(),
                self.data_qualifier[i].to_string(),
                self.flow_direction[i].to_string(),
                self.kind[i].to_string(),
                self.phase[i].to_string(),
                self.uom[i].to_string(),
            ])
            .map_err(|x| x.to_string())?;
        }
        let csv = String::from_utf8(wtr.into_inner().map_err(|x| x.to_string())?).unwrap();
        return Ok(csv);
    }

    #[wasm_bindgen(js_name = "asParquet")]
    pub fn as_parquet(&self) -> Result<Vec<u8>, String> {
        // Originally authored when we had logic to convert a timeseries to arrow.
        // We used the export to arrow, converted the arrow to parquet, and then
        // pulled the schema off the parquet file via
        // https://docs.rs/parquet/latest/parquet/schema/printer/index.html/
        let message_type = "
            message arrow_schema {
                REQUIRED BYTE_ARRAY title (STRING);
                REQUIRED FLOAT cost;
                REQUIRED BYTE_ARRAY quality (STRING);
                REQUIRED FLOAT value;
                REQUIRED INT32 tou;
                REQUIRED INT64 time_period_start_unix_ms (TIMESTAMP(MILLIS, false));
                REQUIRED INT32 time_period_duration_seconds;
                REQUIRED BYTE_ARRAY accumulation_behaviour (STRING);
                REQUIRED BYTE_ARRAY commodity (STRING);
                REQUIRED BYTE_ARRAY currency (STRING);
                REQUIRED BYTE_ARRAY data_qualifier (STRING);
                REQUIRED BYTE_ARRAY flow_direction (STRING);
                REQUIRED BYTE_ARRAY kind (STRING);
                REQUIRED BYTE_ARRAY phase (STRING);
                REQUIRED BYTE_ARRAY uom (STRING);
            }
        ";
        let schema = Arc::new(parse_message_type(message_type).unwrap());
        let props = Arc::new(
            WriterProperties::builder()
                .set_compression(parquet::basic::Compression::SNAPPY)
                .build(),
        );
        let buf: Vec<u8> = vec![];
        let mut tracked_write: TrackedWrite<_> = TrackedWrite::new(buf);

        let mut writer = SerializedFileWriter::new(&mut tracked_write, schema, props).unwrap();
        {
            // Order must match the schema.
            let mut row_group_writer = writer.next_row_group().unwrap();
            // Title is a special case, since it's Strings, not strs.
            write_strs(
                &mut row_group_writer,
                &self.title.iter().map(|x| x.as_str()).collect::<Vec<_>>(),
            )?;
            write_f32s(&mut row_group_writer, &self.cost)?;
            write_strs(&mut row_group_writer, &self.quality)?;
            write_f32s(&mut row_group_writer, &self.value)?;
            write_i32s(&mut row_group_writer, &self.tou)?;
            write_i64s(&mut row_group_writer, &self.time_period_start_unix_ms)?;
            write_i32s(&mut row_group_writer, &self.time_period_duration_seconds)?;
            write_strs(&mut row_group_writer, &self.accumulation_behaviour)?;
            write_strs(&mut row_group_writer, &self.commodity)?;
            write_strs(&mut row_group_writer, &self.currency)?;
            write_strs(&mut row_group_writer, &self.data_qualifier)?;
            write_strs(&mut row_group_writer, &self.flow_direction)?;
            write_strs(&mut row_group_writer, &self.kind)?;
            write_strs(&mut row_group_writer, &self.phase)?;
            write_strs(&mut row_group_writer, &self.uom)?;
            row_group_writer.close().map_err(|x| x.to_string())?;
        }
        writer.close().unwrap();

        return Ok(tracked_write.inner().to_vec());
    }

    #[wasm_bindgen(js_name = "asInfluxdb")]
    pub fn as_influxdb(&self) -> String {
        let mut result = String::new();
        let special_chars = Regex::new(r"[^A-Za-z0-9_]").unwrap();
        let has_cost = self.has_cost();

        for i in 0..self.value.len() {
            let measurement_without_spaces = &self.title[i].replace(" ", "_");
            let measurement = special_chars.replace_all(&measurement_without_spaces, "");
            let tags = vec![
                "db=greenbutton".to_owned(),
                format!(
                    "accumulation_behavior={}",
                    self.accumulation_behaviour[i].replace(" ", "\\ ")
                ),
                format!("commodity={}", self.commodity[i].replace(" ", "\\ ")),
                format!("currency={}", self.currency[i].replace(" ", "\\ ")),
                format!(
                    "data_qualifier={}",
                    self.data_qualifier[i].replace(" ", "\\ ")
                ),
                format!(
                    "flow_direction={}",
                    self.flow_direction[i].replace(" ", "\\ ")
                ),
                format!("kind={}", self.kind[i].replace(" ", "\\ ")),
                format!("phase={}", self.phase[i].replace(" ", "\\ ")),
                format!("uom={}", self.uom[i].replace(" ", "\\ ")),
            ]
            .join(",");

            let mut fields = vec![
                format!("quality={}", self.quality[i].replace(" ", "\\ ")),
                format!("value={}", self.value[i]),
                format!("tou={}", self.tou[i]),
                format!(
                    "time_period_duration_seconds={}",
                    self.time_period_duration_seconds[i]
                ),
            ];
            if has_cost {
                fields.push(format!("cost={}", self.cost[i]));
            }
            let fields = fields.join(",");

            let time_ns = self.time_period_start_unix_ms[i] * 1000000;
            result += &format!("{measurement},{tags} {fields} {time_ns}\n");
        }
        return result.to_owned();
    }
}

#[cfg(target_arch = "wasm32")]
fn vec_str_to_vec_string(x: &[&'static str]) -> Vec<String> {
    return x.iter().map(|x| x.to_string()).collect();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl TimeSeries {
    #[wasm_bindgen(getter)]
    pub fn title(&self) -> Vec<String> {
        return self.title.clone();
    }
    #[wasm_bindgen(getter)]
    pub fn cost(&self) -> Vec<f32> {
        return self.cost.clone();
    }
    #[wasm_bindgen(getter)]
    pub fn quality(&self) -> Vec<String> {
        return vec_str_to_vec_string(&self.quality);
    }
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> Vec<f32> {
        return self.value.clone();
    }
    #[wasm_bindgen(getter)]
    pub fn tou(&self) -> Vec<i32> {
        return self.tou.clone();
    }
    // time_period_start_unix_ms handled below,
    // as we specify the typescript manually for it.
    #[wasm_bindgen(getter)]
    pub fn time_period_duration(&self) -> Vec<i32> {
        return self.time_period_duration_seconds.clone();
    }
    #[wasm_bindgen(getter)]
    pub fn accumulation_behaviour(&self) -> Vec<String> {
        return vec_str_to_vec_string(&self.accumulation_behaviour);
    }
    #[wasm_bindgen(getter)]
    pub fn commodity(&self) -> Vec<String> {
        return vec_str_to_vec_string(&self.commodity);
    }
    #[wasm_bindgen(getter)]
    pub fn currency(&self) -> Vec<String> {
        return vec_str_to_vec_string(&self.currency);
    }
    #[wasm_bindgen(getter)]
    pub fn data_qualifier(&self) -> Vec<String> {
        return vec_str_to_vec_string(&self.data_qualifier);
    }
    #[wasm_bindgen(getter)]
    pub fn flow_direction(&self) -> Vec<String> {
        return vec_str_to_vec_string(&self.flow_direction);
    }
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> Vec<String> {
        return vec_str_to_vec_string(&self.kind);
    }
    #[wasm_bindgen(getter)]
    pub fn phase(&self) -> Vec<String> {
        return vec_str_to_vec_string(&self.phase);
    }
    #[wasm_bindgen(getter)]
    pub fn uom(&self) -> Vec<String> {
        return vec_str_to_vec_string(&self.uom);
    }

    #[wasm_bindgen(getter, skip_typescript, js_name = "time_period_start")]
    pub fn time_period_start_unix_ms(&self) -> js_sys::Array {
        return self
            .time_period_start_unix_ms
            .iter()
            .map(|x| js_sys::Date::from(chrono::DateTime::from_timestamp(*x / 1000, 0).unwrap()))
            .collect::<js_sys::Array>();
    }
}

// Manually add the date getter to the interface.
#[wasm_bindgen(typescript_custom_section)]
const STEP_TYPES: &str = r###"
export interface TimeSeries {
	time_period_start: Date[];
}
"###;

#[cfg(test)]
mod tests {
    use super::TimeSeries;

    fn get_test_timeseries() -> TimeSeries {
        return TimeSeries {
            title: vec!["a".to_string(), "b".to_string()],
            cost: vec![1.0, 2.0],
            quality: vec!["a", "b"],
            value: vec![3.0, 4.0],
            tou: vec![1, 2],
            time_period_start_unix_ms: vec![3, 4],
            time_period_duration_seconds: vec![3, 4],
            accumulation_behaviour: vec!["a", "b"],
            commodity: vec!["a", "b"],
            currency: vec!["a", "b"],
            data_qualifier: vec!["a", "b"],
            flow_direction: vec!["a", "b"],
            kind: vec!["a", "b"],
            phase: vec!["a", "b"],
            uom: vec!["a", "b"],
        };
    }

    #[test]
    fn as_parquet() {
        let test = get_test_timeseries();
        let parquet = test.as_parquet().unwrap();
        assert_eq!(parquet.len(), 4669);
    }
}
