use rand::thread_rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use rules::CellTemp;
use std::io::prelude::*;
use tile::iterate_rectangle;
mod cell;
pub mod coords;
mod rules;
pub mod tile;
pub use coords::{Direction, DirectionSet};
pub mod world1;

use cell::CellTypes;
pub use cell::{Cell, CellType, CellTypeRef, GrowDirection};
pub use tile::{Tile, SIZE};

pub struct World {
    pub cells: Tile<Cell>,
    // mut name2idx: HashMap<&str, u8>,
    pub types: cell::CellTypes,
    rng: Pcg32,
}

impl World {
    pub fn new() -> World {
        let rng = Pcg32::from_rng(thread_rng()).unwrap();
        World {
            cells: Tile::new(Default::default()),
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

        let cells_temp: Tile<CellTemp> = self
            .cells
            .iter_cells()
            .map(|&cell| rules::prepare_step(types, &mut rng, cell))
            .collect();

        self.cells = cells_temp
            .iter_radius_1()
            .map(|(temp, neighbours)| rules::execute_step(types, &mut rng, temp.cell, neighbours))
            .collect();
    }

    pub fn set_cell(&mut self, pos: coords::Cube, cell: Cell) {
        self.cells.set_cell(pos, cell);
    }

    pub fn get_cell(&self, pos: coords::Cube) -> Cell {
        self.cells.get_cell(pos)
    }

    pub fn get_cells_rectangle(&self) -> Vec<Cell> {
        let pos = coords::Cube { x: 0, y: 0 };
        iterate_rectangle(pos, SIZE as i32, SIZE as i32)
            .map(|coord| self.get_cell(coord))
            .collect()
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

    pub fn iter_cells(&self) -> impl ExactSizeIterator<Item = &Cell> {
        self.cells.iter_cells()
    }

    pub fn iter_cells_with_neighbours(&self) -> tile::NeighbourIter<Cell> {
        self.cells.iter_radius_1()
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
