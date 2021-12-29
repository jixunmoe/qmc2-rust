mod utils;

use qmc2_crypto as qmc2;
use qmc2_crypto::detection::Detection;
use qmc2_crypto::errors::DetectionError;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen(js_name = Detection)]
pub struct DetectionWrapper(Detection);

#[wasm_bindgen]
pub fn get_detection_size() -> usize {
    qmc2::detection::RECOMMENDED_DETECTION_SIZE
}

#[wasm_bindgen(catch)]
pub fn detect(buf: &[u8]) -> Result<DetectionWrapper, JsValue> {
    qmc2::detection::detect(buf)
        .map(DetectionWrapper)
        .map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b - 1
}
