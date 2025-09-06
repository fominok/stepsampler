use std::io::Cursor;

use js_sys::Array;
use stepsampler::step_sample;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn process_files(buffers: JsValue) -> Result<Vec<u8>, JsValue> {
    let arr: Array = buffers.into();

    step_sample(
        arr.iter()
            .map(|js_val| Cursor::new(js_sys::Uint8Array::new(&js_val).to_vec())),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}
