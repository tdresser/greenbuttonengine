use arrow::{
    array::{ArrayRef, Date64Array, Float32Array, Int32Array, RecordBatch, StringArray},
    error::ArrowError,
};
use parquet::{arrow::ArrowWriter, basic::Compression, file::properties::WriterProperties};
use regex::Regex;
use wasm_bindgen::prelude::wasm_bindgen;

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

// Based on https://www.reddit.com/r/rust/comments/mfjiqc/comment/gsx76mb/?utm_source=share&utm_medium=web3x&utm_name=web3xcss&utm_term=1&utm_content=share_button
pub fn reorder<T>(arr: &mut [T], mut index: Vec<usize>) {
    assert_eq!(arr.len(), index.len());
    for i in 0..index.len() {
        let mut left = i;
        let mut right = index[i];

        while right != i {
            arr.swap(left, right);
            index[left] = left;
            left = right;
            right = index[right];
        }
        index[left] = left;
    }
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
        let mut indices: Vec<_> = (0..self.value.len()).collect();
        indices.sort_unstable_by(|i, j| {
            self.title[*i]
                .cmp(&self.title[*j])
                .then(self.time_period_start_unix_ms[*i].cmp(&self.time_period_start_unix_ms[*j]))
        });

        // TODO: try to get rid of these copies.
        reorder(&mut self.title, indices.clone());
        reorder(&mut self.cost, indices.clone());
        reorder(&mut self.quality, indices.clone());
        reorder(&mut self.value, indices.clone());
        reorder(&mut self.tou, indices.clone());
        reorder(&mut self.time_period_start_unix_ms, indices.clone());
        reorder(&mut self.time_period_duration_seconds, indices.clone());
        reorder(&mut self.accumulation_behaviour, indices.clone());
        reorder(&mut self.commodity, indices.clone());
        reorder(&mut self.currency, indices.clone());
        reorder(&mut self.data_qualifier, indices.clone());
        reorder(&mut self.flow_direction, indices.clone());
        reorder(&mut self.kind, indices.clone());
        reorder(&mut self.phase, indices.clone());
        reorder(&mut self.uom, indices.clone());
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

    pub fn as_record_batch(self) -> Result<RecordBatch, ArrowError> {
        return RecordBatch::try_from_iter(vec![
            (
                "title",
                std::sync::Arc::new(StringArray::from(self.title)) as ArrayRef,
            ),
            (
                "cost",
                std::sync::Arc::new(Float32Array::from(self.cost)) as ArrayRef,
            ),
            (
                "quality",
                std::sync::Arc::new(StringArray::from(self.quality)) as ArrayRef,
            ),
            (
                "value",
                std::sync::Arc::new(Float32Array::from(self.value)) as ArrayRef,
            ),
            (
                "tou",
                std::sync::Arc::new(Int32Array::from(self.tou)) as ArrayRef,
            ),
            (
                "time_period_start_unix_ms",
                std::sync::Arc::new(Date64Array::from(self.time_period_start_unix_ms)) as ArrayRef,
            ),
            (
                "time_period_duration",
                std::sync::Arc::new(Int32Array::from(self.time_period_duration_seconds))
                    as ArrayRef,
            ),
            (
                "accumulation_behaviour",
                std::sync::Arc::new(StringArray::from(self.accumulation_behaviour)) as ArrayRef,
            ),
            (
                "commodity",
                std::sync::Arc::new(StringArray::from(self.commodity)) as ArrayRef,
            ),
            (
                "currency",
                std::sync::Arc::new(StringArray::from(self.currency)) as ArrayRef,
            ),
            (
                "data_qualifier",
                std::sync::Arc::new(StringArray::from(self.data_qualifier)) as ArrayRef,
            ),
            (
                "flow_direction",
                std::sync::Arc::new(StringArray::from(self.flow_direction)) as ArrayRef,
            ),
            (
                "kind",
                std::sync::Arc::new(StringArray::from(self.kind)) as ArrayRef,
            ),
            (
                "phase",
                std::sync::Arc::new(StringArray::from(self.phase)) as ArrayRef,
            ),
            (
                "uom",
                std::sync::Arc::new(StringArray::from(self.uom)) as ArrayRef,
            ),
        ]);
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
        let batch = self.clone().as_record_batch().map_err(|x| x.to_string())?;
        let mut buf = Vec::<u8>::new();
        {
            let mut writer = arrow::csv::Writer::new(&mut buf);
            writer.write(&batch).unwrap();
        }
        let csv = String::from_utf8(buf).unwrap();
        return Ok(csv);
    }

    #[wasm_bindgen(js_name = "asParquet")]
    pub fn as_parquet(&self) -> Result<Vec<u8>, String> {
        let batch = self.clone().as_record_batch().map_err(|x| x.to_string())?;
        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();
        let mut buf = Vec::<u8>::new();
        {
            let mut writer = ArrowWriter::try_new(&mut buf, batch.schema(), Some(props)).unwrap();
            writer.write(&batch).unwrap();
            writer.close().map_err(|x| x.to_string())?;
        }
        return Ok(buf);
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

fn vec_str_to_vec_string(x: &[&'static str]) -> Vec<String> {
    return x.iter().map(|x| x.to_string()).collect();
}

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
}

// In general, we're not bothering to strip wasm related functions
// from non wasm builds, but this one has a build failure I don't understand,
// so we'll strip it.
// TODO: clean this up.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl TimeSeries {
    #[wasm_bindgen(getter, skip_typescript, js_name = "time_period_start")]
    pub fn time_period_start_unix_ms(&self) -> js_sys::Array {
        return self
            .time_period_start_unix_ms
            .iter()
            .map(|x| js_sys::Date::from(chrono::DateTime::from_timestamp(*x / 1000, 0).unwrap()))
            .collect::<js_sys::Array>();
    }
}

// Manually add the method to the interface.
#[wasm_bindgen(typescript_custom_section)]
const STEP_TYPES: &str = r###"
export interface TimeSeries {
	time_period_start: Date[];
}
"###;
