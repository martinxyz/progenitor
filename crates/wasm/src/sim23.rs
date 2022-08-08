use wasm_bindgen::prelude::*;

use crate::JsSimulation;

use progenitor::{blobs, sim2, tumblers, turing};

#[wasm_bindgen]
pub fn demo_sim2() -> JsSimulation {
    let sim = sim2::World::new();
    sim.into()
}

#[wasm_bindgen]
pub fn demo_turing() -> JsSimulation {
    let sim = turing::Turing::new();
    sim.into()
}

#[wasm_bindgen]
pub fn demo_tumblers() -> JsSimulation {
    // 0.17 approximately maximizes the score for 50 iterations
    let sim = tumblers::Tumblers::new(0.17);
    sim.into()
}

#[wasm_bindgen]
pub fn demo_moving_blobs() -> JsSimulation {
    let sim = blobs::Blobs::new();
    sim.into()
}
