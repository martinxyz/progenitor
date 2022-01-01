#![feature(array_zip)]
use progenitor::world1::Params;
use progenitor::{world1, World};
use rand::{thread_rng, Rng};
use rayon::prelude::*;

use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;

use crate::features::FEATURE_COUNT;

mod features;

fn run(params: &Params) -> World {
    let mut world = World::new();
    world.types = world1::rules(params);
    let iterations = 10;
    for _ in 0..iterations {
        world.tick();
    }
    world
}

fn sample_initial_params(rng: &mut impl Rng) -> Params {
    let mut p = Params::default();
    for _ in 0..64 {
        p.mutate(rng);
    }
    p
}

fn process(params: &Params) -> ([f64; FEATURE_COUNT], World) {
    let repetitions = 32;
    let score: [f64; FEATURE_COUNT] = features::evaluate(|| run(params), repetitions);
    eprintln!("{:?}", params);
    println!("{:.6} {:.6}", score[0], score[1]);
    let world = run(params);
    (score, world)
}

fn main() {
    #[cfg(debug_assertions)]
    {
        eprintln!("warning: was compiled without optimizations");
    }

    const POPULATION_LAG: usize = 100;
    const EVALUATIONS: usize = 10_000;
    // const INITIAL_POPULATION: usize = 10_000;

    let mut rng = thread_rng();

    let mut bins_found = HashMap::new();
    crossbeam::thread::scope(|s| {
        let (tasks_s, tasks_r) = crossbeam::channel::unbounded();
        let (results_s, results_r) = crossbeam::channel::unbounded();

        // add initial tasks
        for i in 0..POPULATION_LAG {
            let params = sample_initial_params(&mut rng);
            tasks_s.send((i, params)).unwrap();
        }

        // process tasks (in background)
        s.spawn(move |_| {
            tasks_r.into_iter().par_bridge().for_each(|(i, params)| {
                results_s.send((i, process(&params))).unwrap();
            });
        });

        // consume each result, add task that depend on this result
        let mut ready_results = HashMap::new();
        for i in 0..EVALUATIONS {
            while !ready_results.contains_key(&i) {
                let (i, res) = results_r.recv().unwrap();
                ready_results.insert(i, res);
                if ready_results.len() > POPULATION_LAG / 2 {
                    eprintln!(
                        "Warning: LAG too low? ready_results / LAG: {} / {}",
                        ready_results.len(),
                        POPULATION_LAG
                    );
                }
            }
            let (score, world) = ready_results.remove(&i).unwrap();

            let params = sample_initial_params(&mut rng);
            let task_i = i + POPULATION_LAG;
            if task_i < EVALUATIONS {
                tasks_s.send((task_i, params)).unwrap();
            }

            let map_resolution = (0.05, 0.02);
            let bc1 = (score[0] / map_resolution.0).round() as i32;
            let bc2 = (score[1] / map_resolution.1).round() as i32;
            let bin = (bc1, bc2);
            bins_found.insert(bin, world);
        }
    })
    .unwrap();
    eprintln!("found {} bins: {:?}", bins_found.len(), bins_found.keys());

    let snapshots: Vec<((i32, i32), Vec<u8>)> = bins_found
        .into_iter()
        .map(|(bin, world)| (bin, world.export_snapshot()))
        .collect();
    create_dir_all("output").unwrap();
    let mut file = File::create("output/map_bins.dat").unwrap();
    let data = bincode::serialize(&snapshots).unwrap();
    file.write_all(&data).unwrap();
}
