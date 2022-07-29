use wasm_bindgen::prelude::*;

use crate::JsSimulation;

use progenitor::{sim2, tumblers, turing};

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
    let sim = tumblers::Tumblers::new();
    sim.into()
}
