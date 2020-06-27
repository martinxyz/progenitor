use crate::coords;
use crate::{CellType, CellTypeRef, SIZE};
pub use hex2d::{Coordinate, Direction};
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
#[allow(dead_code)] // it's actually not unused
pub fn get_size() -> u32 {
    SIZE
}

#[wasm_bindgen]
// pub needed?
pub struct World {
    inner: crate::World,
    data: Vec<u8>,
}

#[wasm_bindgen]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new() -> World {
        set_panic_hook();
        World {
            inner: crate::World::new(),
            data: vec![0; (SIZE * SIZE) as usize],
        }
    }

    pub fn make_some_cells(&mut self) {
        self.inner.types.add_type(&CellType {
            air_like: false,
            child_type: CellTypeRef(2), // numeric pointer!!! very bad as an API
            ..CellType::default()
        });
        self.inner.types.add_type(&CellType {
            air_like: false,
            child_type: CellTypeRef(1), // numeric pointer!!! very bad as an API
            ..CellType::default()
        });
    }

    pub fn set_cell(&mut self, col: i32, row: i32, ct: u8) {
        let pos = coords::Offset { col, row };
        self.inner
            .set_cell(pos.into(), self.inner.types.create_cell(CellTypeRef(ct)));
    }

    pub fn tick(&mut self, direction: i32) {
        self.inner.tick(Direction::from_int(direction));
    }

    pub fn update_data(&mut self) -> Uint8Array {
        /*
        self.data = self.inner.cells
            .iter_cells()
            .map(|c| c.get_type().0)
        */
        self.inner.get_cell_types(self.data.as_mut_slice());

        // JS constructor will copy the data
        Uint8Array::from(self.data.as_slice())
        // zero-copy:
        // unsafe {
        //     // if no rust code runs before the data is used it is safe, if I understand this correctly
        //     Uint8Array::view(self.data.as_slice())
        // }
    }

    /* enabling this makes wasm_opt crash (sig11)
    // which is strange, because it's the previous content of update_data(), I just renamed
    pub fn count_cells(&self) -> usize {
        let count_growing_cells = |w: &crate::World| {
            self.inner.cells.iter_cells().filter(|c| c.get_type().0 != 0).count()
        };
        count_growing_cells(&self.inner)
    }
    */
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
