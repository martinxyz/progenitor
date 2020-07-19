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

    pub fn set_rules_demo1(&mut self) {
        let c1 = CellTypeRef(1);
        let c2 = CellTypeRef(2);
        self.inner.types[c1] = CellType {
            priority: 1,
            max_children: 255,
            child_type: c2,
            ..CellType::default()
        };
        self.inner.types[c2] = CellType {
            priority: 1,
            max_children: 255,
            child_type: c1,
            ..CellType::default()
        };
    }

    pub fn set_rules_demo2(&mut self) {
        let types = &mut self.inner.types;
        // Very loosely based on Zupanc et al., 2019: "Stochastic cellular automata model
        // of tumorous neurosphere growth: Roles of developmental maturity and cell death"

        let empty = CellTypeRef(0);
        types[empty] = CellType {
            priority: -1, // cells with priority 0 may replace "empty" cells with their children
            ..CellType::default()
        };

        let stem_cell = CellTypeRef(1);
        let progenitor_cell = CellTypeRef(2);
        let differentiated_cell = CellTypeRef(3);
        types[stem_cell] = CellType {
            max_children: 255,
            child_type: progenitor_cell,
            // child_condition: always,
            ..CellType::default()
        };
        types[progenitor_cell] = CellType {
            max_children: 7,
            child_type: differentiated_cell,
            ..CellType::default()
        };
    }

    pub fn set_cell(&mut self, col: i32, row: i32, ct: u8) {
        let pos = coords::Offset { col, row };
        self.inner
            .set_cell(pos.into(), self.inner.types.create_cell(CellTypeRef(ct)));
    }

    pub fn tick(&mut self) {
        self.inner.tick();
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
