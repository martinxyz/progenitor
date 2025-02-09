use rand::thread_rng;
use wasm_bindgen::prelude::*;

use crate::JsSimulation;

use progenitor::{
    builders, falling_sand, growth, hive::HiveSim, pairs, sunburn, tumblers, turing, Simulation,
};

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
    let sim = tumblers::Tumblers::new();
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

#[wasm_bindgen]
pub fn demo_sunburn() -> JsSimulation {
    let sim = sunburn::SunburnWorld::new();
    sim.into()
}

#[wasm_bindgen]
pub fn demo_pairs() -> JsSimulation {
    let sim = pairs::World::new(pairs::random_params(&mut thread_rng()));
    // let sim = pairs::World::new(pairs::Params {
    //     p0: 211,
    //     p1: 102,
    //     p2: 203,
    //     count0: 2,
    //     count1: 1,
    // });

    sim.into()
}

#[wasm_bindgen]
pub fn demo_growth() -> JsSimulation {
    growth::GrowthSim::new().into()
}

#[wasm_bindgen]
pub fn demo_growth_default_config() -> String {
    serde_json::to_string(&growth::Configuration::default()).unwrap()
}
#[wasm_bindgen]
pub fn demo_growth_with_config(config: &str) -> Result<JsSimulation, JsValue> {
    let config: growth::Configuration = serde_json::from_str(config)
        .map_err(|e| JsValue::from(format!("Could not parse Configuration:\n {}", e)))?;
    Ok(config.into_simulation().into())
    // serde_wasm_bindgen::from_value(object)  // advantage: also deserializes arrays, hashmaps, etc. (but slower, and dependency)
}

#[wasm_bindgen]
pub fn demo_hive(seeds: &[u64]) -> JsSimulation {
    HiveSim::new_with_seeds(seeds).into()
}

#[wasm_bindgen]
pub fn measure_hive(seeds: &[u64]) -> Vec<f32> {
    let mut sim = HiveSim::new_with_seeds(seeds);
    sim.steps(1000);
    let mut m1 = 0.0;
    let mut m2 = 0.0;
    let n = 100;
    for _ in 0..n {
        sim.steps(10);
        let size = sim.measure_size();
        m1 += size;
        m2 += sim.measure_edges() / size;
    }
    [m1/(n as f32), m2/(n as f32)].into()
}
