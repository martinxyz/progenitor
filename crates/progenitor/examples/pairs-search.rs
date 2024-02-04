use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};

use indicatif::ParallelProgressIterator;
use progenitor::pairs;
use rayon::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(debug_assertions)]
    {
        eprintln!("warning: was compiled without optimizations");
    }

    const EVALUATIONS: usize = 10_000;
    println!("Starting optimization...");

    let result: Vec<_> = (0..=255)
        .into_par_iter()
        .progress_count(256u64)
        .map(|p0| {
            let mut score = 0f32;
            let mut sim = pairs::World::new();
            for _ in 0..EVALUATIONS {
                score += sim.run_and_evaluate(p0);
            }
            // println!("{} score {}", p0, score);
            (p0, score)
        })
        .collect();
    println!("Done.");

    create_dir_all("output")?;
    let filename = "output/pairs-search.csv";
    let mut stream = BufWriter::new(File::create(filename)?);

    writeln!(stream, "p0,score")?;
    for (p0, score) in result {
        writeln!(stream, "{},{}", p0, score)?;
    }
    stream.flush()?;
    println!("Result written to {}", filename);

    Ok(())
}
