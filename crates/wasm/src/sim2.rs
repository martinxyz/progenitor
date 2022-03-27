use wasm_bindgen::prelude::*;

use crate::JsSimulation;

use progenitor::sim2;

#[wasm_bindgen]
pub fn demo_sim2() -> JsSimulation {
    let sim = sim2::World::new();
    sim.into()
}
