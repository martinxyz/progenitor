use progenitor::world1::Params;
use progenitor::{CellTypeRef, World, world1};
use rand::{thread_rng, Rng};
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

fn evaluate(params: &Params) -> f64 {
    let world = run(params);
    let empty = CellTypeRef(1);
    let n = world.iter_cells().filter(|&c| c.cell_type != empty).count();
    // println!("{} cells after {} iterations", n, iterations);
    (n as f64) / (world.iter_cells().count() as f64)
}

fn main() {
    #[cfg(debug_assertions)]
    {
        println!("warning: was compiled without optimizations");
    }

    const POPULATION_SIZE: usize = 10_000;

    let mut rng = thread_rng();
    let population: Vec<Params> = (0..POPULATION_SIZE)
        .map(|_| {
            let mut p = Params::default();
            p.mutate(&mut rng);
            p
        })
        .collect();

    let scale_to_int = 10.0;

    let bins_found = population
        .into_par_iter()
        .map(|params| {
            let repetitions = 32;
            let score = (0..repetitions)
                .map(|_| evaluate(&params))
                .sum::<f64>()
                / repetitions as f64;
            println!("score: {:.6}, params: {:?}", score, params);
            let world = run(&params);

            let bin = (score * scale_to_int).round() as i32;
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

    for (bin, world) in bins_found {
        let filename = format!("output_{}.dat", bin as f64 / scale_to_int);
        println!("found bin {}, writing {}", bin, filename);
        let data = world.export_snapshot();
        let mut file = File::create(&filename).unwrap();
        file.write_all(&data).unwrap();
    }
}
