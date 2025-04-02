use progenitor::{rainfall::RainfallSim, Simulation};
// use rayon::prelude::*;
use std::time::Instant;

pub fn measure_rainfall(seeds: &[u64]) -> (f32, f32) {
    let mut sim = RainfallSim::new_with_seeds(seeds);
    sim.steps(50);
    // let m1 = sim.measure_size();
    sim.steps(300 - 50);
    let m1 = sim.measure_size();
    let m2 = sim.measure_edges() / m1;
    (m1, m2)
}

fn main() {
    #[cfg(debug_assertions)]
    {
        eprintln!("warning: was compiled without optimizations");
    }
    eprintln!("test");

    let t0 = Instant::now();
    let n = 1000;
    let results: Vec<_> = (0..n)
        // .into_par_iter()
        .into_iter()
        .map(|_| measure_rainfall(&[0]))
        .collect();
    let mut result = (0.0, 0.0);
    for tmp in results {
        result.0 += tmp.0;
        result.1 += tmp.1;
    }
    let elapsed = t0.elapsed();
    println!(
        "result: {:.6}, {:.6}",
        result.0 / n as f32,
        result.1 / n as f32
    );
    println!(
        "elapsed: {:.3?}, {:.3?} evals/s",
        elapsed,
        n as f32 / elapsed.as_secs_f32()
    );
}
