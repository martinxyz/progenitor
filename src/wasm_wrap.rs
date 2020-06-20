use wasm_bindgen::prelude::*;
use js_sys::Uint8Array;
pub use hex2d::{Direction, Coordinate};
use crate::SIZE;

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
#[allow(dead_code)]  // it's actually not unused
pub fn get_size() -> u32 {
    SIZE
}

#[wasm_bindgen]
pub struct World {  // pub needed?
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
            data: vec![0; (SIZE*SIZE) as usize],
        }
    }

    pub fn make_some_cells(&mut self) {
        use crate::{CellType, CellTypeRef, Coordinate};
        let w = &mut self.inner;
        let growing_cell = w.types.add_type(&CellType{
            air_like: false,
            child_type: CellTypeRef(1),  // self-pointer !!! very bad API
            ..CellType::default()
        });
        let pos1 = Coordinate::new(0, 0);
        w.cells.set_cell(pos1, w.types.create_cell(growing_cell));
    }

    pub fn tick(&mut self) {
        self.inner.tick(Direction::XY);
    }

    pub fn update_data(&mut self) -> Uint8Array {
        /*
        self.data = self.inner.cells
            .iter_cells()
            .map(|c| c.get_type().0)
            .collect();
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
