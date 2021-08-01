#![feature(array_zip)]
use progenitor::world1::Params;
use progenitor::{Cell, CellTypeRef, World, world1};
use rand::thread_rng;
use rayon::prelude::*;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

fn run(params: &Params) -> World {
    let mut world = World::new();
    world.types = world1::rules(params);
    let iterations = 10;
    for _ in 0..iterations {
        world.tick();
    }
    world
}

#[derive(Default, Clone, Copy)]
struct FeatureAccumulator {
    acc: i64,
    n: u32,
}

impl FeatureAccumulator {
    fn push(&mut self, value: i32) {
        self.acc += value as i64;
        self.n += 1;
    }
    fn push_weighted(&mut self, value: i32, weight: u16) {
        self.acc += value as i64;
        self.n += weight as u32;
    }
    fn merge((fa1, fa2): (Self, Self)) -> Self {
        Self {
            acc: fa1.acc + fa2.acc,
            n: fa1.n + fa2.n,
        }
    }
}

impl From<FeatureAccumulator> for f64 {
    fn from(fa: FeatureAccumulator) -> Self {
        fa.acc as f64 / fa.n as f64
    }
}

const EMPTY: CellTypeRef = CellTypeRef(1);

fn calculate_features(world: World) -> [FeatureAccumulator; 2] {
    fn cell2int(c: Cell) -> i32 {
        if c.cell_type == EMPTY { 0 } else { 1 }
    }
    let mut features = [FeatureAccumulator::default(); 2];
    for (cell, neighbours) in world.iter_cells_with_neighbours() {
        let center = cell2int(cell);
        let neighbours: i32 = neighbours.iter().map(|(_, c)| cell2int(*c)).sum();
        features[0].push(center);
        features[1].push_weighted((neighbours - 6 * center).abs(), 6);
    }
    features
}

fn evaluate(params: &Params) -> [FeatureAccumulator; 2] {
    let repetitions = 32;
    (0..repetitions)
        .map(|_| run(params))
        .map(calculate_features)
        .reduce(|a, b| a.zip(b).map(FeatureAccumulator::merge))
        .unwrap()
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
            let score: [f64; 2] = evaluate(&params).map(|v| v.into());
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

    /*
    for (bin, world) in bins_found {
        let filename = format!("output_{}_{}.dat", bin.0, bin.1);
        eprintln!("found bin {:?}, writing {}", bin, filename);
        let data = world.export_snapshot();
        let mut file = File::create(&filename).unwrap();
        file.write_all(&data).unwrap();
    }
    */

    let snapshots: Vec<((i32, i32), Vec<u8>)> = bins_found
        .into_iter()
        .map(|(bin, world)| (bin, world.export_snapshot()))
        .collect();
    let mut file = File::create("output/map_bins.dat").unwrap();
    let data = bincode::serialize(&snapshots).unwrap();
    file.write_all(&data).unwrap();
}
