use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand_pcg::Pcg32;
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
        World {
            cells: Tile::new(),
            types: CellTypes::new(),
            rng: Pcg32::seed_from_u64(0),
        }
    }

    pub fn tick(&mut self) {
        let types = &self.types;
        let mut rng = &mut self.rng;
        self.cells.mutate_with_radius_1(|cell, _neighbours| {
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

                // 2. self-transformation (independent from neighbours)
                let cell_type = &types[cell.cell_type];
                if let Some(value1) = cell_type.transform_at_value1 {
                    if cell.value1 == value1 {
                        *cell = types.create_cell(cell_type.transform_into);
                    }
                }
            });
            // should probably not put all the logic in here
            // if let Some(value1) = cell_type.transform_at_value1 {
            //     if cell.value1 == value1 {
            //         self.cells.set_cell(x, y, self.types.create_cell(cell_type.transform_into))
            //     }
            // }
        }

        self.cells.mutate_with_radius_1(|cell, _neighbours| {
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
