mod cell;
mod rules;

use crate::coords;
use crate::CellView;
use crate::HexgridView;
use crate::SimRng;
use crate::Simulation;
use crate::TorusNeighbourIter;
use crate::TorusTile;
use crate::VIEWPORT;
use rand::prelude::*;
use std::io::prelude::*;

pub use cell::{Cell, CellType, CellTypeRef, CellTypes, GrowDirection};

pub struct World {
    pub cells: TorusTile<Cell>,
    // mut name2idx: HashMap<&str, u8>,
    pub types: cell::CellTypes,
    rng: SimRng,
}

impl World {
    pub fn new() -> World {
        let rng = SimRng::from_rng(&mut rand::rng());
        World {
            cells: TorusTile::new(Default::default()),
            types: CellTypes::new(),
            rng,
        }
    }

    pub fn seed(&mut self, seed: u64) {
        self.rng = SimRng::seed_from_u64(seed);
    }

    pub fn set_cell(&mut self, pos: coords::Cube, cell: Cell) {
        self.cells.set_cell(pos, cell);
    }

    pub fn cell(&self, pos: coords::Cube) -> Cell {
        self.cells.cell(pos)
    }

    pub fn iter_cells(&self) -> impl ExactSizeIterator<Item = &Cell> {
        self.cells.iter_cells()
    }

    pub fn iter_cells_with_neighbours(&self) -> TorusNeighbourIter<Cell> {
        self.cells.iter_radius_1()
    }
}

impl Simulation for World {
    fn step(&mut self) {
        let types = &self.types;
        let mut rng = &mut self.rng;

        let cells_temp: TorusTile<rules::CellTemp> = self
            .cells
            .iter_cells()
            .map(|&cell| rules::prepare_step(types, &mut rng, cell))
            .collect();

        self.cells = cells_temp
            .iter_radius_1()
            .map(|nh| rules::execute_step(types, &mut rng, nh))
            .collect();
    }

    fn save_state(&self) -> Vec<u8> {
        let mut res = vec![1u8]; // version to signal breaking changes
        res.append(&mut bincode::serialize(&self.rng).unwrap());
        res.append(&mut bincode::serialize(&self.cells).unwrap());
        res
    }

    fn load_state(&mut self, data: &[u8]) {
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
}

impl HexgridView for World {
    fn cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        Some(self.cell(pos).into())
    }

    fn viewport_hint(&self) -> coords::Rectangle {
        VIEWPORT
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
