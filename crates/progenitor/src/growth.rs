use std::array;

use hex2d::Angle;
use rand::thread_rng;
use rand::Rng;
use rand::RngCore;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

use crate::coords;
use crate::hexmap;
use crate::AxialTile;
use crate::CellView;
use crate::Direction;
use crate::DirectionSet;
use crate::HexgridView;
use crate::SimRng;
use crate::Simulation;

const RADIUS: i32 = 21;
const MAX_CELL_TYPES: u8 = 8;
const GROWTH_REQURIEMENT: u16 = 12;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Configuration {
    initial_energy: u16,
    cell_types: u8,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            initial_energy: 8000,
            cell_types: 6,
        }
    }
}

impl Configuration {
    pub fn into_simulation(self) -> GrowthSim {
        assert!(self.cell_types <= MAX_CELL_TYPES);
        let seed = thread_rng().next_u64();
        GrowthSim::new_with_config(seed, self)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Cell {
    rule: u8,
    connections: DirectionSet,
    energy: u16,
}

impl Cell {
    const EMPTY: Cell = Cell {
        rule: 0,
        energy: 0,
        connections: DirectionSet::none(),
    };
}

#[derive(Serialize, Deserialize)]
pub struct GrowthSim {
    rng: SimRng,
    // config: Configuration,
    rules: [CellType; MAX_CELL_TYPES as usize],
    state: AxialTile<Option<Cell>>,
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
            flow: array::from_fn(|_| rng.gen_range(0..=4)),
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

    fn flow(&self, connections: DirectionSet, flow_dir: Direction) -> u8 {
        let c1: usize = connections.contains(flow_dir + Angle::RightBack).into();
        let c2: usize = connections.contains(flow_dir + Angle::Back).into();
        let c3: usize = connections.contains(flow_dir + Angle::LeftBack).into();
        // More fun, harder to comprehend?:
        // let c1: usize = connections.contains(flow_dir + Angle::Left).into();
        // let c2: usize = connections.contains(flow_dir + Angle::Forward).into();
        // let c3: usize = connections.contains(flow_dir + Angle::Right).into();
        let idx = (c1 << 2) | (c2 << 1) | (c3 << 0);
        self.flow[idx]
    }
}

impl GrowthSim {
    pub fn new() -> Self {
        // let mut best_seed = 0;
        // let mut best_score = i32::MIN;
        // for _ in 0..50 {
        //     let seed = thread_rng().next_u64();
        //     let mut trial = Self::new_with_seed(seed);
        //     let score = {
        //         trial.steps(800);
        //         let c1 = trial.count_cells();
        //         let score1 = -(trial.count_cells() - 200).pow(2);
        //         trial.steps(300);
        //         let score2 = -(trial.count_cells() - c1).pow(2) * 100;
        //         score1 + score2
        //     };
        //     if score > best_score {
        //         best_score = score;
        //         best_seed = seed;
        //     }
        // }
        Self::new_with_config(thread_rng().next_u64(), Configuration::default())
    }

    fn new_with_config(seed: u64, config: Configuration) -> Self {
        let mut rng = SimRng::seed_from_u64(seed);
        let state = hexmap::new(RADIUS, None, |location| {
            Some({
                if location.dist_from_center() == 0 {
                    Cell {
                        rule: 1,
                        energy: config.initial_energy,
                        // less symmetry:
                        connections: DirectionSet::single(Direction::West),
                        // hex-symmetrical growth:
                        // connections: DirectionSet::none(),
                    }
                } else {
                    Cell::EMPTY
                }
            })
        });
        let rules = array::from_fn(|i| {
            if i == 0 {
                CellType::new_inert()
            } else {
                CellType::new_random(&mut rng, &config)
            }
        });
        Self { rng, rules, state }
    }

    fn count_cells(&self) -> i32 {
        self.state
            .iter_cells()
            .filter(|c| c.map_or(false, |c| c.rule != 0))
            .count() as i32
    }
}

impl Simulation for GrowthSim {
    fn step(&mut self) {
        self.state = self.state.ca_step(None, |neighbourhood| {
            let Some(neighbourhood) = neighbourhood.valid_only() else {
                return neighbourhood.center;
            };
            if neighbourhood.center.rule == 0 {
                // no energy flow, but a neighbour may grow a cell here
                let mut grow_into = 0;
                let mut grow_from = DirectionSet::none();
                let mut grow_allowed = false;
                for (dir, neigh) in neighbourhood.iter_dirs() {
                    let neigh_grow_into =
                        self.rules[neigh.rule as usize].grow_type(neigh.connections, -dir);
                    let neigh_grow_allowed = neigh.energy >= GROWTH_REQURIEMENT;
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
                if grow_allowed {
                    Some(Cell {
                        rule: grow_into,
                        energy: 0,
                        connections: grow_from,
                    })
                } else {
                    Some(neighbourhood.center)
                }
            } else {
                // only energy flow, no transformations
                //
                // Energy transfer happens between two cells. It is based only
                // on information that both cells can see. To keep things
                // simple, we allow a cell to transfer energy away only if it
                // could transfer the same amount to all 6 neighbours, otherwise
                // it could transfer more than it has.
                //
                let mut energy_transfer: i32 = 0;
                let center = neighbourhood.center;
                let center_rule = &self.rules[center.rule as usize];
                for (dir, neigh) in neighbourhood.iter_dirs() {
                    let neigh_rule = &self.rules[neigh.rule as usize];
                    let flow = {
                        let flow1 = center_rule.flow(center.connections, -dir);
                        let flow2 = neigh_rule.flow(neigh.connections, dir);
                        u8::min(flow1, flow2)
                    };

                    let energy1 = center.energy;
                    let energy2 = neigh.energy;

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
                let mut energy: i32 = center.energy.into();
                energy += energy_transfer;
                assert!(energy >= 0);
                let energy = energy.clamp(0, 0xFFFF) as u16;
                Some(Cell { energy, ..center })
            }
        });
    }

    fn save_state(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        *self = bincode::deserialize_from(data).unwrap();
    }
}

impl HexgridView for GrowthSim {
    fn cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        let cell: Cell = self.state.cell(pos)??;
        Some(CellView {
            cell_type: cell.rule,
            direction: None,
            energy: Some(cell.energy.clamp(0, 255) as u8),
        })
    }
    fn cell_text(&self, pos: coords::Cube) -> Option<String> {
        let cell = self.state.cell(pos)??;
        Some(format!("{cell:?}"))
    }
    fn viewport_hint(&self) -> coords::Rectangle {
        hexmap::viewport(RADIUS)
    }
}