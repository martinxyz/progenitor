use rand::thread_rng;
use wasm_bindgen::prelude::*;

use crate::JsSimulation;
use progenitor::sim1;
use progenitor::{coords, world1, SIZE};
use sim1::{CellType, CellTypeRef, GrowDirection};
use world1::Params;

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
    sim.into()
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
    sim.into()
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
    sim.into()
}

#[wasm_bindgen]
pub fn demo_map() -> JsSimulation {
    let mut sim = progenitor_world_empty();
    let mut params = Params::default();
    params.mutate(&mut thread_rng());
    sim.types = world1::rules(&params);
    sim.into()
}

#[wasm_bindgen]
pub struct Snapshots(Vec<((i32, i32), Vec<u8>)>);

#[wasm_bindgen]
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
