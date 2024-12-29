use once_cell::sync::Lazy;
use personalgreenbutton::{parse_xml, TimeSeries};
use std::{mem, sync::Mutex};
use wasm_bindgen::prelude::wasm_bindgen;

static ALL_TIMESERIES: Lazy<Mutex<TimeSeries>> = Lazy::new(|| Mutex::new(TimeSeries::default()));

#[wasm_bindgen]
pub fn parse_xml_perf_test(s: String) -> Result<(), String> {
    parse_xml(&s).map_err(|x| x.to_string())?;
    return Ok(());
}

#[wasm_bindgen]
pub fn ingest_xml(s: &str, path: &str) -> Result<(), String> {
    let mut mutex = ALL_TIMESERIES.lock().map_err(|x| x.to_string())?;
    let new = parse_xml(s).map_err(|x| x.to_string());
    match new {
        Ok(x) => {
            let mut timeseries = mem::take(&mut (*mutex));
            timeseries.extend(x);
            let _ = mem::replace(&mut *mutex, timeseries);
        }
        Err(err) => return Err(format!("Failed to read {}. {}", path, err)),
    }
    return Ok(());
}

#[wasm_bindgen]
pub fn get_timeseries() -> Result<TimeSeries, String> {
    let mut mutex = ALL_TIMESERIES.lock().map_err(|x| x.to_string())?;
    mutex.sort();
    return Ok(mutex.clone());
}

// Split by title.
#[wasm_bindgen]
pub fn get_timeseries_chunked() -> Result<Vec<TimeSeries>, String> {
    let mutex = ALL_TIMESERIES.lock().map_err(|x| x.to_string())?;
    let chunked = mutex.clone().sort_and_chunk();
    return Ok(chunked);
}

#[wasm_bindgen(start)]
pub fn start() -> () {
    // print pretty errors in wasm https://github.com/rustwasm/console_error_panic_hook
    console_error_panic_hook::set_once();
}
