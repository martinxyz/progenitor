use rand::thread_rng;
use std::borrow::BorrowMut;
use std::rc::Rc;

use crate::world1::Params;
pub use hex2d::{Coordinate, Direction};
use progenitor::sim1;
use progenitor::Simulation;
use progenitor::{coords, world1, SIZE};
use sim1::{CellType, CellTypeRef, GrowDirection};
// use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn get_size() -> u32 {
    SIZE
}

#[wasm_bindgen]
pub fn is_debug_build() -> bool {
    cfg!(debug_assertions)
}

fn progenitor_world_empty() -> sim1::World {
    sim1::World::new()
}

fn progenitor_world_with_seeds() -> sim1::World {
    let mut sim = progenitor_world_empty();
    let positions = [(0, 0), (3, 0), (1, -8), (3, -2)];
    let seed_cell = sim.types.create_cell(CellTypeRef(1));
    for (x, y) in positions {
        const C: i32 = (SIZE / 2) as i32;
        let pos = coords::Offset {
            col: C + x,
            row: C + y,
        };
        sim.set_cell(pos.into(), seed_cell);
    }
    sim
}

#[wasm_bindgen]
pub fn demo_simple() -> JsSimulation {
    let mut sim = progenitor_world_with_seeds();

    let c1 = CellTypeRef(1);
    let c2 = CellTypeRef(2);
    sim.types[c1] = CellType {
        priority: 110,
        grow_p: 128,
        grow_child_type: c2,
        ..CellType::default()
    };
    sim.types[c2] = CellType {
        priority: 110,
        grow_p: 128,
        grow_child_type: c1,
        ..CellType::default()
    };
    JsSimulation(Rc::new(sim))
}

#[wasm_bindgen]
pub fn demo_progenitor() -> JsSimulation {
    let mut sim = progenitor_world_with_seeds();
    let types = &mut sim.types;
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
        priority: 50,
        grow_p: 128,
        grow_dir: GrowDirection::RandomChoice,
        // transaction_move_parent_p: 35,
        transform_at_random_p: 2,
        transform_into: empty,
        ..CellType::default()
    };
    types[stem_cell] = CellType {
        priority: 100,
        // max_children: 255,
        grow_child_type: progenitor_cell,
        transform_at_random_p: 0,
        ..base
    };
    types[progenitor_cell] = CellType {
        priority: 40,
        // initial_energy: 6,
        grow_child_type: progenitor_cell,
        transform_into: differentiated_cell,
        transform_at_random_p: 70,
        ..base
    };
    types[differentiated_cell] = CellType {
        grow_p: 0, // XXX required to prevent leaking energy into air (FIXME)
        // e.g. we could either call grow_t somethig else ("friend_t?") and decouple it from growth. Or just disallow setting a grow_p without also setting a grow_type (e.g. proper typing... with option, disallow using the grow_type if it implies zero growth)
        transform_at_random_p: 2,
        ..base
    };
    JsSimulation(Rc::new(sim))
}

#[wasm_bindgen]
pub fn demo_blobs() -> JsSimulation {
    let mut sim = progenitor_world_with_seeds();
    let types = &mut sim.types;

    let mut ref_iterator = (0..255u8).map(CellTypeRef);
    let mut new_ref = || ref_iterator.next().unwrap();

    let genesis = new_ref();
    let air = new_ref();
    let wall = new_ref();
    let pre_wall = new_ref();

    types[genesis] = CellType {
        priority: -128,
        transform_at_random_p: 128,
        transform_into: air,
        grow_child_type: pre_wall,
        grow_p: 2,
        ..CellType::default()
    };

    types[air] = CellType {
        priority: -1,
        ..CellType::default()
    };

    types[pre_wall] = CellType {
        priority: 19,
        // initial_energy: 2,
        transform_at_random_p: 128,
        transform_into: wall,
        grow_child_type: wall,
        grow_p: 80,
        ..CellType::default()
    };
    types[wall] = CellType {
        priority: 20,
        // initial_energy: 2,
        ..CellType::default()
    };
    JsSimulation(Rc::new(sim))
}

#[wasm_bindgen]
pub fn demo_map() -> JsSimulation {
    let mut sim = progenitor_world_empty();
    let mut params = Params::default();
    params.mutate(&mut thread_rng());
    sim.types = world1::rules(&params);
    JsSimulation(Rc::new(sim))
}

#[wasm_bindgen(js_name = Simulation)]
pub struct JsSimulation(Box<dyn Simulation>);

#[wasm_bindgen(js_class = Simulation)]
impl JsSimulation {
    // pub fn set_cell(&mut self, col: i32, row: i32, ct: u8) {
    //     let pos = coords::Offset { col, row };
    //     self.set_cell(pos.into(), self.types.create_cell(CellTypeRef(ct)));
    // }

    pub fn get_cell_info(&self, col: i32, row: i32) -> JsValue {
        let pos = coords::Offset { col, row };
        let cell = self.0.get_cell_view(pos.into());
        JsValue::from_serde(&cell).unwrap()
    }

    pub fn step(&mut self) {
        self.0.borrow_mut().step();
    }

    pub fn get_data(&mut self, channel: u8) -> Vec<u8> {
        self.0
            .get_cells_rectangle()
            .iter()
            .map(|cell| match channel {
                0 => cell.cell_type,
                1 => cell.energy.unwrap_or(0),
                2 => match cell.direction {
                    Some(dir) => dir as u8,
                    None => 0, // better API contract required...
                },
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
        self.0.save_state()
    }

    pub fn import_snapshot(&mut self, data: &[u8]) {
        self.0.load_state(data);
    }
}

// #[wasm_bindgen]
// pub struct Snapshot {
//     bins: [i32; 2],
//     pub data: Vec<u8>,
// }

#[wasm_bindgen]
pub struct Snapshots(Vec<((i32, i32), Vec<u8>)>);

// #[wasm_bindgen]
// fn deserializeSnapshot()

#[wasm_bindgen]
// there probably is a way to transfer the whole thing a once instead...
impl Snapshots {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Self {
        Self(bincode::deserialize(data).unwrap_or_default())
    }

    // that works:
    // pub fn getall(&self) -> Box<[JsValue]> {
    //     let data: Box<[JsValue]> = self.0.iter().map(|&((a, _), _)| JsValue::from(a)).collect();
    //     data
    // }
    pub fn getall(&self) -> Box<[JsValue]> {
        let data: Box<[JsValue]> = self
            .0
            .iter()
            .map(|((f1, f2), data)| {
                let arr = js_sys::Array::new();
                arr.push(&(*f1).into());
                arr.push(&(*f2).into());
                let data: js_sys::Uint8Array = data.as_slice().into();
                arr.push(&data);
                arr.into()
            })
            .collect();
        data
    }
}
