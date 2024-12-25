#[derive(PartialEq, Clone, Copy, Serialize, Deserialize, Debug)]
pub enum Cell {
    Border,
    Air,
    Wall,
    Hive,
    Worker,
    Plant,
    Fruit,
}
use serde::{Deserialize, Serialize};
use Cell::*;

use crate::{
    coords, tiled::load_axial_tile_from_json, AxialTile, CellView, HexgridView, Simulation,
};

#[derive(Serialize, Deserialize)]
pub struct HiveSim {
    cells: AxialTile<Cell>,
}

pub fn new() -> HiveSim {
    static JSON: &str = include_str!("../../../../maps/testmap.tmj");
    let map = load_axial_tile_from_json(JSON, Border, |idx| match idx {
        0 => Border,
        2 => Wall,
        6 => Air,
        10 => Hive,
        13 => Plant,
        16 => Fruit,
        _ => Border,
    });
    HiveSim { cells: map }
}

impl Simulation for HiveSim {
    fn step(&mut self) {}

    fn save_state(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        *self = bincode::deserialize_from(data).unwrap();
    }
}

impl HexgridView for HiveSim {
    fn cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        let cell_type = match self.cells.cell(pos)? {
            Border => return None,
            Air => 2,
            Wall => 4,
            Hive => 5,
            Worker => 0,
            Plant => 3,
            Fruit => 1,
        };
        Some(CellView {
            cell_type,
            ..Default::default()
        })
    }

    fn viewport_hint(&self) -> coords::Rectangle {
        self.cells.viewport()
    }
}
