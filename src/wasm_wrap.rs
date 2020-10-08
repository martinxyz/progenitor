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
            transaction_child_type: c2,
            ..CellType::default()
        };
        self.inner.types[c2] = CellType {
            priority: 1,
            max_children: 255,
            transaction_child_type: c1,
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
            transaction_skip_p: 255, // 120
            transaction_move_parent_p: 35,
            transform_at_random_p: 10,
            transform_into: empty,
            ..CellType::default()
        };
        types[stem_cell] = CellType {
            max_children: 255,
            transaction_child_type: progenitor_cell,
            transform_at_random_p: 0,
            ..base
        };
        types[progenitor_cell] = CellType {
            max_children: 7,
            transaction_child_type: differentiated_cell,
            ..base
        };
        types[differentiated_cell] = CellType {
            transform_at_random_p: 2,
            ..base
        };
    }

    pub fn set_rules_demo3(&mut self) {
        let types = &mut self.inner.types;

        // let params = [207, 7, 51, 250];
        // let params = [91, 33, 181, 66];
        // let params = [2, 126, 177, 148];
        let params = [202, 0, 52, 10];

        let empty = CellTypeRef(0);
        types[empty] = CellType {
            priority: -1, // cells with priority 0 may replace "empty" cells with their children
            ..CellType::default()
        };

        let stem_cell = CellTypeRef(1);
        let progenitor_cell = CellTypeRef(2);
        let differentiated_cell = CellTypeRef(3);
        let interior_dead_cell = CellTypeRef(4);
        let slime = CellTypeRef(5);
        let base = CellType {
            transaction_skip_p: 255, // 120
            transaction_move_parent_p: params[0],
            transform_at_random_p: params[1],
            transform_into: interior_dead_cell,
            ..CellType::default()
        };
        types[stem_cell] = CellType {
            max_children: 255,
            transaction_child_type: progenitor_cell,
            transform_at_random_p: 0,
            ..base
        };
        types[progenitor_cell] = CellType {
            max_children: params[2],
            transaction_child_type: differentiated_cell,
            ..base
        };
        types[differentiated_cell] = CellType {
            max_children: 255,
            transaction_child_type: slime, // why does it seem to move when producing slime?
            // skip_transaction_p: 120,
            transaction_skip_p: 0,
            transaction_move_parent_p: 0,

            transform_at_random_p: params[3],
            ..base
        };
        types[slime] = CellType {
            priority: -1, // cells with priority 0 may replace "slime" cells with their children
            transform_at_random_p: 1,
            transform_into: empty,
            ..CellType::default()
        };
        types[interior_dead_cell] = CellType {
            transform_into: slime,
            ..types[slime]
        };

        // let cell = types.create_cell(stem_cell);
        // world.set_cell(coords::Cube { x: 5, y: 5 }, cell);
        // world
    }

    pub fn set_rules_demo4(&mut self) {
        let types = &mut self.inner.types;

        let empty = CellTypeRef(0);
        let photon = CellTypeRef(1);
        types[empty] = CellType {
            priority: -50, // cells with priority 0 may replace "empty" cells with their children
            transform_at_random_p: 1,
            transform_into: photon,
            ..CellType::default()
        };
        types[photon] = CellType {
            priority: -1,
            // transform_into: CellTypeRef(2),
            transaction_move_parent_p: 128,
            max_children: 255,
            transaction_child_type: CellTypeRef(0),
            // transform_at_value1: Some(0),
            transaction_skip_p: 129,
            ..CellType::default()
        };

        // idea: some kind of "algae"/"photoreceptor" cell; it doesn't need to
        // "belong" to another entity, but is required to provide energy to
        // other blobs. So they are kind of herded / farmed / grown next to.
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

    pub fn export_snapshot(&self) -> Vec<u8> {
        // FIXME: this gets converted to a JS Uint8Array, but:
        // I think (hope) it's a copy, not a reference to the wasm memory. XXX
        // (Also, why didn't I do it that way for update_data()? Was it just
        // a premature optimization to avoid the copy?)
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
