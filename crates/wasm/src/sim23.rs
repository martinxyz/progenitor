use wasm_bindgen::prelude::*;

use crate::JsSimulation;

use progenitor::{builders, falling_sand, tumblers, turing};

#[wasm_bindgen]
pub fn demo_falling_sand() -> JsSimulation {
    let sim = falling_sand::World::new();
    sim.into()
}

#[wasm_bindgen]
pub fn demo_turing() -> JsSimulation {
    let sim = turing::Turing::new();
    sim.into()
}

#[wasm_bindgen]
pub fn demo_tumblers() -> JsSimulation {
    // 0.3 approximately maximizes the score for 50 iterations (depending on map size, etc.)
    let sim = tumblers::Tumblers::new(0.3);
    sim.into()
}

#[wasm_bindgen]
pub fn demo_builders_random() -> JsSimulation {
    let sim = builders::Builders::new_with_random_params();
    sim.into()
}

#[wasm_bindgen]
pub fn demo_builders_optimized() -> JsSimulation {
    let sim = builders::Builders::new_optimized();
    sim.into()
}
