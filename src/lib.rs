use rand::{SeedableRng, seq::SliceRandom};
#[cfg(not(target_arch = "wasm32"))]
use rand::thread_rng;
use rand_pcg::Pcg32;
use std::io::prelude::*;
mod cell;
pub mod coords;
mod tile;
pub use coords::{Direction, DirectionSet};

#[cfg(all(feature = "python", not(target_arch = "wasm32")))]
mod py_wrap;
// #[cfg(target_arch = "wasm32")]
mod wasm_wrap;

use cell::CellTypes;
pub use cell::{Cell, CellType, CellTypeRef}; // note: Cell should not be pub? at least not its internals
pub use tile::{Tile, SIZE};

pub struct World {
    cells: Tile,
    // mut name2idx: HashMap<&str, u8>,
    pub types: cell::CellTypes,
    rng: Pcg32,
}

/* maybe?
trait CellChannel {
    fn extract(c: Cell) -> u8;
    fn update(c: &mut Cell);
}

struct CellTypeChannel {}

impl CellChannel for CellTypeChannel {
    fn extract(c: Cell) -> u8;
    fn update(c: &mut Cell)
}

pub enum Channels {
    Particles,
    CellTypes,
}
*/

impl World {
    pub fn new() -> World {
        #[cfg(not(target_arch = "wasm32"))]
        let rng = Pcg32::from_rng(thread_rng()).unwrap();
        #[cfg(target_arch = "wasm32")]  // no thread_rng
        let rng = Pcg32::seed_from_u64(0);
        World {
            cells: Tile::new(),
            types: CellTypes::new(),
            rng,
        }
    }

    pub fn seed(&mut self, seed: u64) {
        self.rng = Pcg32::seed_from_u64(seed);
    }

    pub fn tick(&mut self) {
        let types = &self.types;
        let mut rng = &mut self.rng;
        self.cells.mutate_with_radius_1(|cell, _neighbours| {
            *cell = types.self_transform(&mut rng, *cell);
            *cell = types.prepare_transaction(&mut rng, *cell);
        });

        let mut directions: Vec<Direction> = Direction::all().to_vec();
        directions.shuffle(&mut rng);

        for dir in directions {
            self.cells.mutate_with_radius_1(|cell, neighbours| {
                // 1. transactions to/from neighbours
                // note(performance): if we're going to do just one direction at a time, we obviously could do much more efficient interation
                let prev = neighbours[dir as usize];
                let next = neighbours[-dir as usize];
                let t1 = types.get_transaction(prev, *cell, dir);
                let t2 = types.get_transaction(*cell, next, dir);
                *cell = types.execute_transactions(t1, *cell, t2);

                // 2. self-transformation (independent from neighbours, 6 executions per step)
                // currently not used
            });
        }

        self.cells.mutate_with_radius_1(|cell, _neighbours| {
            // we do this a the end, rather than the start, mostly just to allow
            // cell comparision in unit-tests
            *cell = types.clear_transaction(*cell);
        });
    }

    pub fn set_cell(&mut self, pos: coords::Cube, cell: Cell) {
        self.cells.set_cell(pos, cell);
    }

    pub fn get_cell(&self, pos: coords::Cube) -> Cell {
        self.cells.get_cell(pos)
    }

    pub fn get_cell_types(&self, buf: &mut [u8]) {
        let pos = coords::Cube { x: 0, y: 0 };
        let it = Tile::iterate_rectangle(pos, SIZE as i32, SIZE as i32);
        for (idx, coord) in it.enumerate() {
            buf[idx] = self.get_cell(coord).cell_type.0;
        }
    }

    pub fn export_snapshot(&self) -> Vec<u8> {
        let mut res = vec![1u8]; // version to signal breaking changes
        res.append(&mut bincode::serialize(&self.rng).unwrap());
        res.append(&mut bincode::serialize(&self.cells).unwrap());
        res
    }

    pub fn import_snapshot(&mut self, data: &[u8]) {
        let mut unread = data;
        let mut version = [0u8; 1];
        unread.read_exact(&mut version).unwrap();
        if version != [1] {
            panic!("version not compatible")
        }
        self.rng = bincode::deserialize_from(&mut unread).unwrap();
        self.cells = bincode::deserialize_from(&mut unread).unwrap();
        // ignore extra, if any (allows for non-breaking extensions)
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = &Cell> {
        self.cells.iter_cells()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

// #[pymodule]
// fn progenitor(_py: Python, m: &PyModule) -> PyResult<()> {
//     // m.add_wrapped(wrap_pyfunction!(test_rng))?;
//     m.add_class::<cell::CellState>()?;
//     #[pyfn(m, "get_tile_size")]
//     fn get_tile_size() -> i32 { tile::SIZE }
//     Ok(())
// }
