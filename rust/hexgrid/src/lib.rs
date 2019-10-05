use wasm_bindgen::prelude::*;

mod hex;
mod misc;

use hex::coord;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    let coord: coord::AxialCoord = Default::default();
    alert(&format!("Hello, {:?}!", coord));
}
