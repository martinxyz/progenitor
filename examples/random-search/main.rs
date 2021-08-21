#![feature(array_zip)]
use progenitor::world1::Params;
use progenitor::{World, world1};
use rand::thread_rng;
use rayon::prelude::*;

use std::collections::HashMap;
use std::fs::File;
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

fn main() {
    #[cfg(debug_assertions)]
    {
        eprintln!("warning: was compiled without optimizations");
    }

    const POPULATION_SIZE: usize = 10_000;
    // const POPULATION_SIZE: usize = 100;

    let mut rng = thread_rng();
    let population: Vec<Params> = (0..POPULATION_SIZE)
        .map(|_| {
            let mut p = Params::default();
            for _ in 0..64 {
                p.mutate(&mut rng);
            }
            p
        })
        .collect();

    let map_resolution = (0.05, 0.02);

    let bins_found = population
        .into_par_iter()
        .map(|params| {
            let repetitions = 32;
            let score: [f64; FEATURE_COUNT] = features::evaluate(|| run(&params), repetitions);
            eprintln!("{:?}", params);
            println!("{:.6} {:.6}", score[0], score[1]);
            let world = run(&params);

            let bc1 = (score[0] / map_resolution.0).round() as i32;
            let bc2 = (score[1] / map_resolution.1).round() as i32;
            let bin = (bc1, bc2);
            (bin, world)
        })
        .fold(HashMap::new, |mut hs, (bin, world)| {
            hs.insert(bin, world);
            hs
        })
        .reduce(HashMap::new, |mut h1, h2| {
            h1.extend(h2);
            h1
        });

    eprintln!("found {} bins: {:?}", bins_found.len(), bins_found.keys());

    let snapshots: Vec<((i32, i32), Vec<u8>)> = bins_found
        .into_iter()
        .map(|(bin, world)| (bin, world.export_snapshot()))
        .collect();
    let mut file = File::create("output/map_bins.dat").unwrap();
    let data = bincode::serialize(&snapshots).unwrap();
    file.write_all(&data).unwrap();
}
