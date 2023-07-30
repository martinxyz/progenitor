use progenitor::{builders::Builders, Simulation};

fn main() {
    #[cfg(debug_assertions)]
    {
        eprintln!("warning: was compiled without optimizations");
    }

    let mut total = 0.;
    const N: usize = 50;
    for _ in 0..N {
        let mut sim = Builders::new_with_random_params();
        for _ in 0..3000 {
            sim.step();
        }
        total += sim.hoarding_score() as f32;
    }

    println!("score: {:.6}", total / (N as f32))
}
