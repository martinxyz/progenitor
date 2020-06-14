use wasm_bindgen::prelude::*;
pub use hex2d::{Direction, Coordinate};

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
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
        use crate::{CellType, CellTypeRef, Coordinate};
        set_panic_hook();
        let mut w = crate::World::new();
        let growing_cell = w.types.add_type(&CellType{
            air_like: false,
            child_type: CellTypeRef(1),  // self-pointer !!! very bad API
            ..CellType::default()
        });
        let pos1 = Coordinate::new(5, 5);
        w.cells.set_cell(pos1, w.types.create_cell(growing_cell));
        World{ inner: w, data: vec![] }
    }

    pub fn tick(&mut self) {
        self.inner.tick(Direction::XY);
    }

    pub fn update_data(&self) -> usize {
        let count_growing_cells = |w: &crate::World| {
            self.inner.cells.iter_cells().filter(|c| c.get_type().0 == 0).count()
        };
        count_growing_cells(&self.inner)
    }
}
