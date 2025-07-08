use nalgebra::SVector;
use wasm_bindgen::prelude::*;

use crate::JsSimulation;

use progenitor::{
    builders, falling_sand, growth, pairs, rainfall::RainfallSim, sunburn, tumblers, turing,
    Simulation,
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
    let sim = pairs::World::new(pairs::random_params(&mut rand::rng()));
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
pub fn demo_rainfall(seeds: &[u64]) -> JsSimulation {
    RainfallSim::new_with_seeds(seeds).into()
}

#[wasm_bindgen]
pub fn measure_rainfall(seeds: &[u64]) -> Vec<f32> {
    const M: usize = 2;
    const EVALS: usize = 1;
    let mut measures = SVector::<f32, M>::zeros();
    for eval in 0..EVALS {
        let m: SVector<f32, M> = {
            let mut sim = RainfallSim::new_with_seeds(seeds);
            if eval != 0 {
                sim.re_seed(eval as u64);
            }
            sim.steps(600);
            let mut weights: f32 = 0.0;
            let mut m1 = 0.0;
            let mut m2 = 0.0;
            let n = 50;
            for ss in 0..n {
                sim.steps(16);
                let size = sim.measure_size();
                // let weight = 1.0f32;
                let weight = ss as f32;
                let weight = weight * weight;
                m1 += weight * size;
                m2 += weight * sim.measure_edges() / (size + 4.0);
                weights += weight;
            }
            [m1 / weights, m2 / weights].into()
        };
        measures += m;
    }
    measures /= EVALS as f32;
    measures.as_slice().into()
}
