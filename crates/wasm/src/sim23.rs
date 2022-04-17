use wasm_bindgen::prelude::*;

use crate::JsSimulation;

use progenitor::{sim2, turing_drawings};

#[wasm_bindgen]
pub fn demo_sim2() -> JsSimulation {
    let sim = sim2::World::new();
    sim.into()
}

#[wasm_bindgen]
pub fn demo_turing1() -> JsSimulation {
    let sim = turing_drawings::Turing1::new();
    sim.into()
}

#[wasm_bindgen]
pub fn demo_turing2() -> JsSimulation {
    let sim = turing_drawings::Turing2::new();
    sim.into()
}
