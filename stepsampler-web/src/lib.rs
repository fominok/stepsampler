use std::io::Cursor;

use js_sys::Array;
use stepsampler::{Config, DEFAULT_OUT_RATE, DEFAULT_SILENCE_THRESHOLD, step_sample};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn process_files(
    buffers: JsValue,
    rate: Option<u32>,
    silence_threshold: Option<f32>,
    stereo: Option<bool>,
) -> Result<Vec<u8>, JsValue> {
    let arr: Array = buffers.into();

    let config = Config {
        rate: rate.unwrap_or(DEFAULT_OUT_RATE),
        silence_threshold: silence_threshold.unwrap_or(DEFAULT_SILENCE_THRESHOLD),
        stereo: stereo.unwrap_or(false),
    };

    step_sample(
        config,
        arr.iter()
            .map(|js_val| Cursor::new(js_sys::Uint8Array::new(&js_val).to_vec())),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}
