use super::vapour;
use crate::{
    coords, tiled::load_axial_tile_from_json, AxialTile, BitParticles, CellView, Direction,
    DirectionSet, HexgridView, SimRng, Simulation,
};
use hex2d::Angle;
use rand::{thread_rng, Rng, SeedableRng};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use std::{array, fmt::Debug};
use Cell::*;

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
struct Hex {
    cell: Cell,
    vapour: BitParticles,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
enum Cell {
    Border,
    Air,
    Wall,
    Seed,
    Worker,
    Object1,
    Object2,
    Plant(PlantCell),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct PlantCell {
    rule: u8,
    connections: DirectionSet,
    energy: u16,
}

impl PlantCell {
    const EMPTY: PlantCell = PlantCell {
        rule: 0,
        energy: 0,
        connections: DirectionSet::none(),
    };
}

const BORDER: Hex = Hex {
    cell: Border,
    vapour: BitParticles::EMPTY,
};

#[derive(Serialize, Deserialize)]
pub struct HiveSim {
    hexes: AxialTile<Hex>,
    rng: SimRng,
    config: Configuration,
    rules: [CellType; MAX_CELL_TYPES as usize],
}
const MAX_CELL_TYPES: u8 = 8;
const GROWTH_REQURIEMENT: u16 = 12;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Configuration {
    initial_energy: u16,
    initial_symmetry: Symmetry,
    cell_types: u8,
    max_flow: u8,
    flow_swap: bool,
    grow_prob: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Symmetry {
    Unidirectional,
    Broad,
    Bidirectional,
    Triangular,
    Angled,
    Full,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            initial_energy: 8000,
            initial_symmetry: Symmetry::Broad,
            cell_types: 2,
            max_flow: 12,
            flow_swap: false,
            grow_prob: 0.97,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CellType {
    flow: [u8; 8],
    growth: [u8; 8],
}

impl CellType {
    fn new_inert() -> Self {
        Self {
            flow: [0; 8],
            growth: [0; 8],
        }
    }
    fn new_random(rng: &mut impl Rng, config: &Configuration) -> Self {
        Self {
            flow: array::from_fn(|_| rng.gen_range(0..=config.max_flow)),
            growth: array::from_fn(|_| rng.gen_range(0..config.cell_types)),
        }
    }
    fn grow_type(&self, connections: DirectionSet, growth_dir: Direction) -> u8 {
        let c1: usize = connections.contains(growth_dir + Angle::RightBack).into();
        let c2: usize = connections.contains(growth_dir + Angle::Back).into();
        let c3: usize = connections.contains(growth_dir + Angle::LeftBack).into();
        let idx = (c1 << 2) | (c2 << 1) | (c3 << 0);
        self.growth[idx]
    }

    fn flow(&self, connections: DirectionSet, flow_dir: Direction, swap: bool) -> u8 {
        let c1: usize;
        let c2: usize;
        let c3: usize;
        if !swap {
            c1 = connections.contains(flow_dir + Angle::RightBack).into();
            c2 = connections.contains(flow_dir + Angle::Back).into();
            c3 = connections.contains(flow_dir + Angle::LeftBack).into();
        } else {
            // More fun, harder to comprehend?:
            c1 = connections.contains(flow_dir + Angle::Left).into();
            c2 = connections.contains(flow_dir + Angle::Right).into();
            c3 = connections.contains(flow_dir + Angle::Back).into();
        }
        let idx = (c1 << 2) | (c2 << 1) | (c3 << 0);
        self.flow[idx]
    }
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
    let mut rng = Pcg32::from_rng(thread_rng()).unwrap();
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

    let config = Configuration::default();
    let rules = array::from_fn(|i| {
        if i == 0 {
            CellType::new_inert()
        } else {
            CellType::new_random(&mut rng, &config)
        }
    });
    HiveSim {
        hexes: map,
        rng,
        config: Configuration::default(),
        rules,
    }
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

            if let Plant(center_p) = nh.center.cell {
                // only energy flow, no transformations
                //
                // Energy transfer happens between two cells. It is based only
                // on information that both cells can see. To keep things
                // simple, we allow a cell to transfer energy away only if it
                // could transfer the same amount to all 6 neighbours, so
                // it cannot transfer more than it has.
                //
                let mut energy_transfer: i32 = 0;
                let center_rule = &self.rules[center_p.rule as usize];
                for (dir, neigh) in nh.iter_dirs() {
                    let Plant(neigh_p) = neigh.cell else { continue };
                    let neigh_rule = &self.rules[neigh_p.rule as usize];
                    let flow = {
                        let flow1 =
                            center_rule.flow(center_p.connections, -dir, self.config.flow_swap);
                        let flow2 =
                            neigh_rule.flow(neigh_p.connections, dir, self.config.flow_swap);
                        ((flow1 as i16 + flow2 as i16) - 7).clamp(0, 255) as u8
                    };

                    let energy1 = center_p.energy;
                    let energy2 = neigh_p.energy;

                    if energy1 > energy2 {
                        if energy1 >= flow as u16 * 6 {
                            energy_transfer -= flow as i32;
                        }
                    } else if energy2 > energy1 {
                        if energy2 >= flow as u16 * 6 {
                            energy_transfer += flow as i32;
                        }
                    }
                }
                let mut energy: i32 = center_p.energy.into();
                energy += energy_transfer;
                assert!(energy >= 0);
                let energy = energy.clamp(0, 0xFFFF) as u16;
                hex.cell = Plant(PlantCell { energy, ..center_p })
            } else if matches!(nh.center.cell, Air) {
                // no energy flow, but a neighbour may grow a cell here
                let mut grow_into = 0;
                let mut grow_from = DirectionSet::none();
                let mut grow_allowed = false;
                for (dir, neigh) in nh.iter_dirs() {
                    let Plant(neigh_p) = neigh.cell else { continue };
                    let neigh_grow_into =
                        self.rules[neigh_p.rule as usize].grow_type(neigh_p.connections, -dir);
                    let neigh_grow_allowed = neigh_p.energy >= GROWTH_REQURIEMENT;
                    if neigh_grow_into > grow_into {
                        // switch to higher priority rule
                        grow_into = neigh_grow_into;
                        grow_from = DirectionSet::none();
                        grow_allowed = false;
                    }
                    if neigh_grow_into == grow_into {
                        grow_from = grow_from.with(dir, true);
                        grow_allowed = grow_allowed || neigh_grow_allowed;
                    }
                }
                if grow_allowed && self.config.grow_prob < 1.0 {
                    if !self.rng.gen_bool(self.config.grow_prob.into()) {
                        grow_allowed = false
                    }
                }
                if grow_allowed {
                    hex.cell = Plant(PlantCell {
                        rule: grow_into,
                        energy: 0,
                        connections: grow_from,
                    })
                }
            } else if matches!(hex.cell, Seed) {
                hex.cell =
                    Plant(PlantCell {
                        rule: 1,
                        energy: self.config.initial_energy,
                        connections: match self.config.initial_symmetry {
                            Symmetry::Unidirectional => DirectionSet::single(Direction::West),
                            Symmetry::Broad => DirectionSet::single(Direction::West)
                                .with(Direction::NorthWest, true),
                            Symmetry::Angled => DirectionSet::single(Direction::West)
                                .with(Direction::NorthEast, true),
                            Symmetry::Bidirectional => {
                                DirectionSet::single(Direction::West).with(Direction::East, true)
                            }
                            Symmetry::Triangular => DirectionSet::single(Direction::West)
                                .with(Direction::NorthEast, true)
                                .with(Direction::SouthEast, true),
                            Symmetry::Full => DirectionSet::all(),
                        },
                    })
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
            Plant(p) => match p.rule {
                0 => 3,
                _ => 1,
            },
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
