mod utils;

use qmc2_crypto as qmc2;
use qmc2_crypto::detection::Detection;
use qmc2_crypto::QMC2Crypto;
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
pub struct QMC2CryptoWrapper(Box<dyn QMC2Crypto>);

#[wasm_bindgen]
impl QMC2CryptoWrapper {
    #[wasm_bindgen]
    pub fn get_recommended_block_size(&self) -> usize {
        self.0.get_recommended_block_size()
    }

    #[wasm_bindgen]
    pub fn decrypt(&self, offset: usize, buf: &mut [u8]) {
        self.0.decrypt(offset, buf)
    }
}

#[wasm_bindgen(catch)]
pub fn decrypt_factory(ekey: String) -> Result<QMC2CryptoWrapper, JsValue> {
    qmc2::decrypt_factory(ekey.as_str())
        .map(QMC2CryptoWrapper)
        .map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b - 1
}
