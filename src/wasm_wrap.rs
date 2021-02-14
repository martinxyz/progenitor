use crate::coords;
use crate::{CellType, CellTypeRef, SIZE};
pub use hex2d::{Coordinate, Direction};
// use js_sys::Uint8Array;
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
}

#[wasm_bindgen]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new() -> World {
        set_panic_hook();
        World {
            inner: crate::World::new(),
        }
    }

    pub fn set_rules_demo1(&mut self) {
        let c1 = CellTypeRef(1);
        let c2 = CellTypeRef(2);
        self.inner.types[c1] = CellType {
            priority: 110,
            grow_child_type: c2,
            ..CellType::default()
        };
        self.inner.types[c2] = CellType {
            priority: 110,
            grow_child_type: c1,
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
        let base = CellType {
            grow_p: 255, // 120
            // transaction_move_parent_p: 35,
            transform_at_random_p: 10,
            transform_into: empty,
            ..CellType::default()
        };
        types[stem_cell] = CellType {
            // max_children: 255,
            grow_child_type: progenitor_cell,
            transform_at_random_p: 0,
            ..base
        };
        types[progenitor_cell] = CellType {
            initial_energy: 7,
            grow_child_type: differentiated_cell,
            ..base
        };
        types[differentiated_cell] = CellType {
            transform_at_random_p: 2,
            ..base
        };
    }

    pub fn set_cell(&mut self, col: i32, row: i32, ct: u8) {
        let pos = coords::Offset { col, row };
        self.inner
            .set_cell(pos.into(), self.inner.types.create_cell(CellTypeRef(ct)));
    }

    pub fn get_cell_json(&self, col: i32, row: i32) -> String {
        let pos = coords::Offset { col, row };
        let cell = self.inner.get_cell(pos.into());
        serde_json::to_string(&cell).unwrap()
    }

    pub fn tick(&mut self) {
        self.inner.tick();
    }

    pub fn get_data(&mut self, channel: u8) -> Vec<u8> {
        self.inner
            .get_cells_rectangle()
            .iter()
            .map(|cell| match channel {
                0 => cell.cell_type.0,
                1 => cell.energy,
                2 => cell.heading,
                _ => panic!("invalid channel"),
            })
            .collect()
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

    pub fn export_snapshot(&self) -> Vec<u8> {
        // FIXME: this gets converted to a JS Uint8Array, but:
        // I think (hope) it's a copy, not a reference to the wasm memory. XXX
        self.inner.export_snapshot()
    }

    pub fn import_snapshot(&mut self, data: &[u8]) {
        self.inner.import_snapshot(data);
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
