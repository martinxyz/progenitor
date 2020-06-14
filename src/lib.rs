mod cell;
mod tile;

#[cfg(all(feature = "python", not(target_arch = "wasm32")))]
mod py_wrap;
// #[cfg(target_arch = "wasm32")]
mod wasm_wrap;

pub use tile::{
    Tile,
    SIZE,
};
use cell::CellTypes;
pub use cell::{CellType, CellTypeRef, Cell};  // note: Cell should not be pub? at least not its internals

pub use hex2d::{Direction, Coordinate};

pub struct World {
    pub cells: Tile,
    // mut name2idx: HashMap<&str, u8>,
    pub types: cell::CellTypes,
}

trait ToIndex {
    fn to_index(&self) -> usize;
}
impl ToIndex for hex2d::Direction {
    fn to_index(&self) -> usize {
        // (hex2d docu specifies range [0, 6) - but a signed int type is enforced)
        // maybe this can be somehow solved in the hex2d crate instead?
        self.to_int::<i32>() as usize
    }
}

impl World {
    pub fn new() -> World {
        World {
            cells: Tile::new(),
            types: CellTypes::new()
        }
    }

    pub fn tick(&mut self, tick_direction: Direction) {
        let types = &self.types;
        self.cells.mutate_with_radius_1(|cell, neighbours| {
            // 1. transactions to/from neighbours
            // note(performance): if we're going to do just one direction at a time, we obviously could do much more efficient interation
            let prev = neighbours[tick_direction as usize];
            let next = neighbours[(-tick_direction).to_index()];
            let t1 = types.get_transaction(prev, *cell);
            let t2 = types.get_transaction(*cell, next);
            *cell = types.execute_transactions(t1, *cell, t2);

            // 2. self-transformation (independent from neighbours)
            let cell_type = types.type_from_cell(*cell);
            if let Some(value1) = cell_type.transform_at_value1 {
                if cell.value1 == value1 {
                    *cell = types.create_cell(cell_type.transform_into);
                }
            }
        });
        // for y in 0..SIZE as i32 {
        //     for x in 0..SIZE as i32 {
        //         // should probably not put all the logic in here
        //         let cell = self.cells.get_cell(x, y);
        //         let cell_type = self.types.type_from_cell(&cell);
        //         if let Some(value1) = cell_type.transform_at_value1 {
        //             if cell.value1 == value1 {
        //                 self.cells.set_cell(x, y, self.types.create_cell(cell_type.transform_into))
        //             }
        //         }
        //     }
        // }
    }
}


// use rand::Rng;
// use rand_pcg::Pcg32;
// #[pyfunction]
// fn test_rng(seed: u64) -> PyResult<i32> {
// use rand::prelude::*;
//     // let mut rng = thread_rng();
//     let mut rng = Pcg32::seed_from_u64(seed);
//     println!("Random bool: {:?}", rng.gen());
//     let x = rng.gen_range(6, 100);
//     Ok(x)
// }

// #[pymodule]
// fn progenitor(_py: Python, m: &PyModule) -> PyResult<()> {
//     // m.add_wrapped(wrap_pyfunction!(test_rng))?;
//     m.add_class::<cell::CellState>()?;
//     #[pyfn(m, "get_tile_size")]
//     fn get_tile_size() -> i32 { tile::SIZE }
//     Ok(())
// }