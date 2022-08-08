use progenitor::tumblers;
use progenitor::Simulation;

fn main() {
    #[cfg(debug_assertions)]
    {
        eprintln!("warning: was compiled without optimizations");
    }

    let max_steps = 32;
    let iterations = 50_000;

    // poor man's numpy.linspace()
    let bins = vec![0.12, 0.13, 0.14, 0.15, 0.16, 0.17, 0.18, 0.19];
    let mut scores = vec![0.; bins.len()];

    for _ in 0..iterations {
        for (bin, &prob) in bins.iter().enumerate() {
            let mut sim = tumblers::Tumblers::new(prob);
            for _ in 0..max_steps {
                sim.step()
            }
            scores[bin] += sim.avg_visited() / (iterations as f32);
        }
    }

    for (value, score) in bins.iter().zip(scores) {
        println!("{} -> {}", value, score);
    }
}
