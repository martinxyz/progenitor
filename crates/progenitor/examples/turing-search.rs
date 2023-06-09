use progenitor::turing::Turing;
use progenitor::{Simulation, SIZE};
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
            // high number of iterations: many bins will be static or oscillating
            // low number of iterations: interesting to play, but most bins look just random
            // widely varied number of iterations: low iteration counts get selected (see above)
            // iterations: 1 << rng.gen_range(12..18),
            iterations: rng.gen_range(10_000..15_000),
        }
    }
    pub fn mutate(&mut self, rng: &mut impl Rng) {
        /*
        // This just tends to converge to small number of iterations (boring).
        let fac_max: f32 = 1.2;
        let fac = rng.gen_range(-fac_max.log2()..fac_max.log2()).exp2();
        self.iterations = ((self.iterations as f32 * fac).clamp(10., 1e6).round()) as u64;
        */
        self.seed = rng.next_u64();
    }
}

fn calculate_features(sim: Turing) -> [FeatureAccumulator; FEATURE_COUNT] {
    let mut features = [FeatureAccumulator::default(); FEATURE_COUNT];

    let mut histogram = [0i64; Turing::SYMBOLS];
    sim.grid.iter_cells().for_each(|&c| {
        histogram[c as usize] += 1;
    });
    let mut sorted: Vec<_> = (0..Turing::SYMBOLS as u8)
        .into_iter()
        .zip(histogram)
        .collect();
    sorted.sort_by_key(|&(_idx, cnt)| -cnt);

    // features[0] measures "how much a single color dominates"
    features[0].push_weighted(
        (sorted[0].1 * sorted[0].1 / ((SIZE * SIZE) as i64)) as i32,
        (SIZE * SIZE) as u16,
    );

    // features[1] measures something related to "number of edges"
    let cell2int = |c: u8| (c != sorted[1].0) as i32;
    for (center, neighbours) in sim.grid.iter_radius_1() {
        let center = cell2int(center);
        let neighbours: i32 = neighbours.iter().map(|(_, c)| cell2int(*c)).sum();
        // sobel edge detector (or similar)
        features[1].push_weighted((neighbours - 6 * center).abs(), 6);
    }
    features
}

pub fn evaluate<F>(run: F) -> [f64; FEATURE_COUNT]
where
    F: Fn() -> Turing,
{
    let features = calculate_features(run());
    features.map(|fa| fa.into())
}
fn run(params: &Params) -> Turing {
    let mut sim = Turing::new_with_seed(params.seed);
    sim.steps(params.iterations.try_into().unwrap());
    sim
}

type EvalResult = ([f64; FEATURE_COUNT], Params, Turing);
fn process(params: Params) -> EvalResult {
    let score: [f64; FEATURE_COUNT] = evaluate(|| run(&params));
    let sim = run(&params);
    (score, params, sim)
}

fn main() {
    #[cfg(debug_assertions)]
    {
        eprintln!("warning: was compiled without optimizations");
    }

    const POPULATION_LAG: usize = 100;
    const EVALUATIONS: usize = 10_000_000;

    let mut rng = thread_rng();

    let mut bins_found: HashMap<_, EvalResult> = HashMap::new();

    let init: Vec<_> = (0..POPULATION_LAG)
        .map(|_| Params::init(&mut rng))
        .collect();
    let mut total_tasks = init.len();
    run_taskstream(init, process, |(i, eval_result)| {
        let map_resolution = (0.025, 0.025);
        let score = eval_result.0;
        let bc1 = (score[0] / map_resolution.0).round() as i32;
        let bc2 = (score[1] / map_resolution.1).round() as i32;
        let bin = (bc1, bc2);

        if bins_found.insert(bin, eval_result).is_none() {
            // print diagnostics:
            let avg_iterations = bins_found
                .values()
                .map(|(_, params, _)| params.iterations as f32)
                .sum::<f32>()
                / (bins_found.len() as f32);
            eprintln!(
                "evaluation {}: found {} bins (avg iterations: {:.1})",
                i,
                bins_found.len(),
                avg_iterations
            );
            println!("{} {}", i, bins_found.len());
        }

        if total_tasks < EVALUATIONS {
            total_tasks += 1;
            let random_parent = bins_found.values().choose(&mut rng).unwrap();
            let mut params = random_parent.1.clone();
            params.mutate(&mut rng);
            // let params = Params::init(&mut rng); // for validation
            Some(params)
        } else {
            None
        }
    });
    eprintln!("found {} bins: {:?}", bins_found.len(), bins_found.keys());

    let snapshots: Vec<((i32, i32), Vec<u8>)> = bins_found
        .into_iter()
        .map(|(bin, (_score, _params, sim))| (bin, sim.save_state()))
        .collect();
    create_dir_all("output").unwrap();
    let mut file = File::create("output/turing_bins.dat").unwrap();
    let data = bincode::serialize(&snapshots).unwrap();
    file.write_all(&data).unwrap();
}
