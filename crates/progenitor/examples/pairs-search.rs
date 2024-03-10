use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};

use indicatif::ParallelProgressIterator;
use progenitor::pairs;
use rand::{thread_rng, SeedableRng};
use rand_pcg::Pcg32;
use rayon::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(debug_assertions)]
    {
        eprintln!("warning: was compiled without optimizations");
    }

    const EVALUATIONS: usize = 1_000;
    println!("Starting optimization...");

    const POPULATION: usize = 4_000;
    let mut result: Vec<_> = (0..POPULATION)
        .into_par_iter()
        .progress_count(POPULATION.try_into().unwrap())
        .map(|_| {
            let mut rng = Pcg32::from_rng(thread_rng()).unwrap();
            let params = pairs::random_params(&mut rng);
            let mut score = 0f32;
            for _ in 0..EVALUATIONS {
                score += pairs::run_and_evaluate(params.clone())
            }
            // println!("{} score {}", p0, score);
            (params, score / EVALUATIONS as f32)
        })
        .collect();
    result.sort_by(|a, b| a.1.partial_cmp(&b.1).expect("score should be valid"));
    println!("Done.");
    println!("Best result:\n{:?}\n", result.last());

    create_dir_all("output")?;
    let filename = "output/pairs-search.csv";
    let mut stream = BufWriter::new(File::create(filename)?);

    writeln!(stream, "score,p0,p1,p2,count0,count1")?;
    for (p, score) in result {
        writeln!(
            stream,
            "{},{},{},{},{},{}",
            score, p.p0, p.p1, p.p2, p.count0, p.count1
        )?;
    }
    stream.flush()?;
    println!("Result written to {}", filename);

    Ok(())
}
