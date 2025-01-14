#[derive(PartialEq, Clone, Copy, Serialize, Deserialize, Debug)]
enum Cell {
    Border,
    Air,
    Wall,
    Seed,
    Worker,
    Object1,
    Object2,
}
use std::fmt::Debug;

use rand::{thread_rng, SeedableRng};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use Cell::*;

use super::vapour;

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
struct Hex {
    cell: Cell,
    vapour: BitParticles,
}

const BORDER: Hex = Hex {
    cell: Border,
    vapour: BitParticles::EMPTY,
};

use crate::{
    coords, tiled::load_axial_tile_from_json, AxialTile, BitParticles, CellView, Direction,
    DirectionSet, HexgridView, SimRng, Simulation,
};

#[derive(Serialize, Deserialize)]
pub struct HiveSim {
    hexes: AxialTile<Hex>,
    rng: SimRng,
}

pub fn new() -> HiveSim {
    static JSON: &str = include_str!("../../../../maps/testmap.tmj");
    let map = load_axial_tile_from_json(JSON, Border, |idx| match idx {
        0 => Border,
        2 => Wall,
        6 => Air,
        10 => Seed,
        13 => Object1,
        16 => Object2,
        _ => Border,
    });
    let rng = Pcg32::from_rng(thread_rng()).unwrap();
    let mut vapour_init = BitParticles::EMPTY;
    vapour_init.set_outgoing(DirectionSet::single(Direction::West));
    let map = map.map(|cell| Hex {
        cell,
        vapour: if matches!(cell, Air) {
            vapour_init
        } else {
            BitParticles::EMPTY
        },
    });

    HiveSim { hexes: map, rng }
}

impl Simulation for HiveSim {
    fn step(&mut self) {
        self.hexes = self.hexes.ca_step(BORDER, |nh| {
            let mut hex = nh.center;
            hex.vapour = vapour::step(nh.map(|h| h.vapour));
            match hex.cell {
                Border | Wall => hex.vapour.reflect_all(),
                _ => vapour::apply_air_rules(&mut hex.vapour, &mut self.rng),
            }
            hex
        });
    }

    fn save_state(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        *self = bincode::deserialize_from(data).unwrap();
    }
}

impl HexgridView for HiveSim {
    fn cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        let hex = self.hexes.cell(pos)?;
        let cell_type = match hex.cell {
            Border => return None,
            Air => 2,
            Wall => 4,
            Seed => 5,
            Worker => 0,
            Object1 => 3,
            Object2 => 1,
        };
        Some(CellView {
            cell_type,
            energy: Some(hex.vapour.outgoing().count() * 2),
            ..Default::default()
        })
    }

    fn cell_text(&self, pos: coords::Cube) -> Option<String> {
        let cell = self.hexes.cell(pos)?;
        Some(format!("{cell:?}"))
    }

    fn viewport_hint(&self) -> coords::Rectangle {
        self.hexes.viewport()
    }
}
