use progenitor::world1::{rules, Cell, CellTypeRef, Params, World};
use progenitor::Simulation;
use rand::prelude::*;

use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;

use utils::{run_taskstream, FeatureAccumulator};

pub const FEATURE_COUNT: usize = 2;

fn calculate_features(world: World) -> [FeatureAccumulator; FEATURE_COUNT] {
    const EMPTY: CellTypeRef = CellTypeRef(1);
    fn cell2int(c: Cell) -> i32 {
        if c.cell_type == EMPTY {
            0
        } else {
            1
        }
    }
    let mut features = [FeatureAccumulator::default(); FEATURE_COUNT];
    for nh in world.iter_cells_with_neighbours() {
        let center = cell2int(nh.center);
        let neighbours: i32 = nh.neighbours.into_iter().map(cell2int).sum();
        features[0].push(center);
        features[1].push_weighted((neighbours - 6 * center).abs(), 6);
    }
    features
}

pub fn evaluate<F>(run: F, repetitions: i32) -> [f64; FEATURE_COUNT]
where
    F: Fn() -> World,
{
    (0..repetitions)
        .map(|_| run())
        .map(calculate_features)
        .reduce(|a, b| {
            [
                FeatureAccumulator::merge(a[0], b[0]),
                FeatureAccumulator::merge(a[1], b[1]),
            ]
        })
        .unwrap()
        .map(|fa| fa.into())
}
fn run(params: &Params) -> World {
    let mut world = World::new();
    world.types = rules(params);
    let iterations = 10;
    for _ in 0..iterations {
        world.step();
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

type EvalResult = ([f64; FEATURE_COUNT], Params, World);
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
    const EVALUATIONS: usize = 10_000;
    // const INITIAL_POPULATION: usize = 10_000;

    let mut rng = rand::rng();

    let mut bins_found: HashMap<_, EvalResult> = HashMap::new();

    let init: Vec<_> = (0..POPULATION_LAG)
        .map(|_| sample_initial_params(&mut rng))
        .collect();
    let mut total_tasks = init.len();
    run_taskstream(init, process, |(i, eval_result)| {
        let map_resolution = (0.05, 0.02);
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
            while rng.random_bool(0.7) {
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
    let mut file = File::create("output/map_bins.dat").unwrap();
    let data = bincode::serialize(&snapshots).unwrap();
    file.write_all(&data).unwrap();
}
