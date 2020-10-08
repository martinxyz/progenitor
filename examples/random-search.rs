use progenitor::{coords, CellType, CellTypeRef, World};
use rayon::prelude::*;
use rand::{Rng, thread_rng};

const PARAM_COUNT: usize = 4;

fn create_world(params: &[u8]) -> World {
    let mut world = World::new();
    let types = &mut world.types;

    let empty = CellTypeRef(0);
    types[empty] = CellType {
        priority: -1, // cells with priority 0 may replace "empty" cells with their children
        ..CellType::default()
    };

    let stem_cell = CellTypeRef(1);
    let progenitor_cell = CellTypeRef(2);
    let differentiated_cell = CellTypeRef(3);
    let interior_dead_cell = CellTypeRef(4);
    let slime = CellTypeRef(5);
    let base = CellType {
        transaction_skip_p: 255, // 120
        transaction_move_parent_p: params[0],
        transform_at_random_p: params[1],
        transform_into: interior_dead_cell,
        ..CellType::default()
    };
    types[stem_cell] = CellType {
        max_children: 255,
        transaction_child_type: progenitor_cell,
        transform_at_random_p: 0,
        ..base
    };
    types[progenitor_cell] = CellType {
        max_children: params[2],
        transaction_child_type: differentiated_cell,
        ..base
    };
    types[differentiated_cell] = CellType {
        max_children: 255,
        transaction_child_type: slime, // why does it seem to move when producing slime?
        // skip_transaction_p: 120,
        transaction_skip_p: 0,
        transaction_move_parent_p: 0,

        transform_at_random_p: params[3],
        ..base
    };
    types[slime] = CellType {
        priority: -1, // cells with priority 0 may replace "slime" cells with their children
        transform_at_random_p: 1,
        transform_into: empty,
        ..CellType::default()
    };
    types[interior_dead_cell] = CellType {
        transform_into: slime,
        ..types[slime]
    };

    let cell = types.create_cell(stem_cell);
    world.set_cell(coords::Cube { x: 5, y: 5 }, cell);
    world
}

fn evaluate(params: &[u8]) -> f64 {
    let mut world = create_world(params);
    let iterations = 100;
    for _ in 0..iterations {
        world.tick();
    }
    let empty = CellTypeRef(0);
    let n = world.iter_cells().filter(|&c| c.cell_type != empty).count();
    // println!("{} cells after {} iterations", n, iterations);
    (n as f64) / (world.iter_cells().count() as f64)
}

fn main() {
    #[cfg(debug_assertions)] {
        println!("warning: was compiled without optimizations");
    }

    let population_size = 10_000;

    let mut rng = thread_rng();
    let population: Vec<Vec<u8>> = (0..population_size).into_iter().map(|_|
        (0..PARAM_COUNT).into_iter()
        .map(|_| rng.gen_range(0, 256) as u8)
        .collect()
    ).collect();

    for params in population {
        let repetitions = 4;
        let score = (0..repetitions)
            .into_par_iter()
            .map(|_| evaluate(&params[0..4]))
            .sum::<f64>() / repetitions as f64;
        println!("score: {:.6}, params: {:?}", score, params);
    }
}
