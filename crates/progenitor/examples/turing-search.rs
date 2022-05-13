#![feature(array_zip)]
// use progenitor::world1::{rules, Cell, CellTypeRef, Params, Turing2};
use progenitor::turing_drawings::Turing2;
use progenitor::Simulation;
use rand::prelude::IteratorRandom;
use rand::{thread_rng, Rng};

use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;

use utils::{run_taskstream, FeatureAccumulator};

pub const FEATURE_COUNT: usize = 2;

#[derive(Clone)]
struct Params {
    seed: u64,
    iterations: u64,
}

impl Params {
    fn init(rng: &mut impl Rng) -> Params {
        Params {
            seed: rng.next_u64(),
            iterations: 1 << rng.gen_range(13..19),
        }
    }
    pub fn mutate(&mut self, rng: &mut impl Rng) {
        let fac_max: f32 = 1.2;
        let fac = rng.gen_range(-fac_max.log2()..fac_max.log2()).exp2();
        self.iterations = ((self.iterations as f32 * fac).clamp(10., 1e6).round()) as u64;

        if rng.gen_bool(0.9) {
            self.seed = rng.next_u64();
        }
    }
}

fn calculate_features(sim: Turing2) -> [FeatureAccumulator; FEATURE_COUNT] {
    let mut features = [FeatureAccumulator::default(); FEATURE_COUNT];

    let mut histogram = [0i64; 256];
    sim.grid.iter_cells().for_each(|&c| {
        histogram[c as usize] += 1;
    });
    // get the most frequent "color" max_c
    let (max_c, _max_count) = (0..=255u8)
        .zip(histogram)
        .max_by_key(|&(_c, count)| count)
        .unwrap();

    let cell2int = |c: u8| (c != max_c) as i32;

    for (center, neighbours) in sim.grid.iter_radius_1() {
        let center = cell2int(center);
        let neighbours: i32 = neighbours.iter().map(|(_, c)| cell2int(*c)).sum();
        features[0].push(center);
        features[1].push_weighted((neighbours - 6 * center).abs(), 6);
    }
    features
}

pub fn evaluate<F>(run: F, repetitions: i32) -> [f64; FEATURE_COUNT]
where
    F: Fn() -> Turing2,
{
    (0..repetitions)
        .map(|_| run())
        .map(calculate_features)
        .reduce(|a, b| a.zip(b).map(FeatureAccumulator::merge))
        .unwrap()
        .map(|fa| fa.into())
}
fn run(params: &Params) -> Turing2 {
    let mut sim = Turing2::new_with_seed(params.seed);
    sim.steps(params.iterations.try_into().unwrap());
    sim
}

type EvalResult = ([f64; FEATURE_COUNT], Params, Turing2);
fn process(params: Params) -> EvalResult {
    let repetitions = 32;
    let score: [f64; FEATURE_COUNT] = evaluate(|| run(&params), repetitions);
    // eprintln!("{:?}", params);
    // println!("{:.6} {:.6}", score[0], score[1]);
    let world = run(&params);
    (score, params, world)
}

fn main() {
    #[cfg(debug_assertions)]
    {
        eprintln!("warning: was compiled without optimizations");
    }

    const POPULATION_LAG: usize = 100;
    const EVALUATIONS: usize = 30_000;

    let mut rng = thread_rng();

    let mut bins_found: HashMap<_, EvalResult> = HashMap::new();

    let init: Vec<_> = (0..POPULATION_LAG)
        .map(|_| Params::init(&mut rng))
        .collect();
    let mut total_tasks = init.len();
    run_taskstream(init, process, |(i, eval_result)| {
        let map_resolution = (0.02, 0.02);
        let score = eval_result.0;
        let bc1 = (score[0] / map_resolution.0).round() as i32;
        let bc2 = (score[1] / map_resolution.1).round() as i32;
        let bin = (bc1, bc2);
        if bins_found.insert(bin, eval_result).is_none() {
            eprintln!("evaluation {}: found {} bins", i, bins_found.len());
            println!("{} {}", i, bins_found.len());
        }

        if total_tasks < EVALUATIONS {
            total_tasks += 1;
            let random_parent = bins_found.values().choose(&mut rng).unwrap();
            let mut params = random_parent.1.clone();
            params.mutate(&mut rng);
            while rng.gen_bool(0.7) {
                params.mutate(&mut rng);
            }
            // params = sample_initial_params(&mut rng);  // for validation: converges ~40% slower
            Some(params)
        } else {
            None
        }
    });
    eprintln!("found {} bins: {:?}", bins_found.len(), bins_found.keys());

    let snapshots: Vec<((i32, i32), Vec<u8>)> = bins_found
        .into_iter()
        .map(|(bin, (_score, _params, world))| (bin, world.save_state()))
        .collect();
    create_dir_all("output").unwrap();
    let mut file = File::create("output/turing_bins.dat").unwrap();
    let data = bincode::serialize(&snapshots).unwrap();
    file.write_all(&data).unwrap();
}
