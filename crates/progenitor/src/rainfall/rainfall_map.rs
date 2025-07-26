use super::symmetric_rule::SymmetricRule;
use super::vapour;
use crate::{
    AxialTile, BitParticles, CellView, HexgridView, SimRng, Simulation, coords,
    tiled::load_axial_tile_from_json,
};
use crate::{DirectionSet, Neighbourhood};
use Cell::*;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::{array, fmt::Debug};

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

impl Cell {
    fn rule(&self) -> u8 {
        match self {
            Plant(p) => p.rule,
            _ => 255,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct PlantCell {
    rule: u8,
    mass: BitParticles,
}

impl PlantCell {
    const EMPTY: PlantCell = PlantCell {
        rule: 0,
        mass: BitParticles::EMPTY,
    };
}

impl Hex {
    fn solid(&self) -> bool {
        matches!(self.cell, Border | Wall | Plant(PlantCell { rule: 3, .. }))
    }
    fn air(&self) -> bool {
        matches!(self.cell, Air)
    }
}

const BORDER: Hex = Hex {
    cell: Border,
    vapour: BitParticles::EMPTY,
};

#[derive(Serialize, Deserialize)]
pub struct RainfallSim {
    hexes: AxialTile<Hex>,
    rng: SimRng,
    config: Configuration,
    rules: [CellType; MAX_CELL_TYPES as usize],
}
const MAX_CELL_TYPES: u8 = 4;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Configuration {
    flow_swap: bool,
}

impl Configuration {
    fn new_random(_rng: &mut impl Rng) -> Self {
        Self { flow_swap: false }
    }

    fn mutate(&mut self, rng: &mut impl Rng) {
        if rng.random::<u8>() < 20 {
            self.flow_swap = rng.random_bool(0.5);
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CellType {
    sr: SymmetricRule,
}

impl CellType {
    fn new_random(rng: &mut impl Rng, _config: &Configuration) -> Self {
        Self {
            sr: SymmetricRule::sample(rng), // grow_prob: sigmoid(rng.random_range(-3.0..3.0f32)),
        }
    }
}

impl RainfallSim {
    pub fn new() -> RainfallSim {
        Self::new_with_seeds(&[rand::rng().next_u64()])
    }

    pub fn re_seed(&mut self, seed: u64) {
        self.rng = SimRng::seed_from_u64(seed);
    }

    pub fn new_with_seeds(seeds: &[u64]) -> RainfallSim {
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
        let mut rng = SimRng::seed_from_u64(seeds[0]);
        let vapour_init = BitParticles::EMPTY;
        // vapour_init.set_outgoing(DirectionSet::single(Direction::West));
        let map = map.map(|cell| Hex {
            cell,
            vapour: if matches!(cell, Air) {
                vapour_init
            } else {
                BitParticles::EMPTY
            },
        });

        let mut config = Configuration::new_random(&mut rng);

        let mut rules = array::from_fn(|_| CellType::new_random(&mut rng, &config));

        // mutations
        let mutation_prob = 0.5 / (rules.len() as f32);
        for &seed2 in &seeds[1..] {
            config.mutate(&mut rng);
            let mut rng = SimRng::seed_from_u64(seed2);
            for rule in rules[1..].iter_mut() {
                if rng.random_bool(mutation_prob.into()) {
                    *rule = CellType::new_random(&mut rng, &config)
                }
            }
        }

        RainfallSim {
            hexes: map,
            rng,
            config,
            rules,
        }
    }
}

impl Simulation for RainfallSim {
    fn step(&mut self) {
        self.hexes = self.hexes.ca_step(BORDER, |nh| {
            let mut next = nh.center;
            next.vapour = vapour::step(nh.map(|h| h.vapour));
            match next.cell {
                Border | Wall => next.vapour.reflect_all(),
                Plant(PlantCell { rule: 2, .. }) => next.vapour.reflect_all(),
                _ => vapour::apply_air_rules(&mut next.vapour, &mut self.rng),
            }

            // check if a neighbour may grow here
            let growth_rule_next = nh.iter_dirs().fold(255, |rule, (dir, neigh)| {
                // neighbour must be a plant
                if let Plant(neigh_p) = neigh.cell {
                    // neighbour must be pushing mass towards us
                    if neigh_p.mass.outgoing().has(-dir) {
                        // neighbour must have the lowest rule of all candidates
                        rule.min(neigh_p.rule)
                    } else {
                        rule
                    }
                } else {
                    rule
                }
            });

            next.cell = if let Plant(mut next_plant) = nh.center.cell {
                // mass transfers to/from neighbouring plant cells
                //
                // (must use only information from the old cell-state of each pair)
                // current rule: mass transfer always happens when possible
                //
                // (if a cell doesn't want to transfer mass out, it has to remove the mass from that direction)
                // (if a cell received mass in that it doesn't want, it will just have to transfer it out again)
                // (however, a transfer out also doubles as a "growth request", so... let's just try and see)

                let nh_view: Neighbourhood<Option<BitParticles>> =
                    nh.map(|neigh| match neigh.cell {
                        Plant(neigh_plant) => {
                            if neigh_plant.rule == next_plant.rule {
                                Some(neigh_plant.mass)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    });

                let rule = &self.rules[next_plant.rule as usize];
                next_plant.mass = rule.sr.step1_transfer(nh_view);
                rule.sr.step2_shuffle(&mut next_plant.mass, &mut self.rng);

                // let next_outgoing = DirectionSet::matching(|dir| {
                //     let outgoing = next_plant.mass.outgoing().has(dir);
                //     match nh[dir].cell {
                //         Plant(p) => {
                //             // swap both cell's outgoing slots
                //             p.mass.outgoing().has(-dir)
                //         }
                //         _ => outgoing,
                //     }
                // });
                // next_plant.mass.set_outgoing(next_outgoing);
                // // randomly distribute mass
                // next_plant.mass.shuffle8_cheap(&mut self.rng);

                // cell death
                // (note: we could allow this, if the cell wants to, with non-zero mass)
                if next_plant.mass.count() == 0 {
                    // same logic as for empty cells (rule_next may be 255 = Air)
                    next_plant = PlantCell {
                        rule: growth_rule_next,
                        mass: BitParticles::EMPTY,
                    }
                }

                // cell that can harvest vapour
                if next_plant.rule == 2 {
                    let vapour_available = nh
                        .neighbours
                        .iter()
                        .map(|n| n.vapour.outgoing().count() as i32)
                        .sum::<i32>();
                    if vapour_available > 0 && next_plant.mass.resting() < 2 {
                        next_plant.mass.set_resting(2);
                        if self.rng.random::<u8>() < 20 {
                            next.vapour.set_outgoing(DirectionSet::none());
                        }
                    }
                }
                if next_plant.rule == 255 {
                    Air
                } else {
                    Plant(next_plant)
                }
            } else if matches!(nh.center.cell, Air) {
                // no mass transfer, but a neighbour may grow an empty cell here
                if growth_rule_next < 255 {
                    Plant(PlantCell {
                        rule: growth_rule_next,
                        mass: BitParticles::EMPTY,
                    })
                } else {
                    Air
                }
            } else if matches!(next.cell, Seed) {
                Plant(PlantCell {
                    // rule: 2,
                    rule: 1,
                    // mass: BitParticles::new(DirectionSet::all(), 2),
                    mass: BitParticles::FULL,
                })
            } else {
                next.cell
            };
            next
        });
    }

    fn save_state(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        *self = bincode::deserialize_from(data).unwrap();
    }
}

impl HexgridView for RainfallSim {
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
                2 => 5,
                3 => 4,
                _ => 1,
            },
        };
        let energy_vapour = hex.vapour.outgoing().count() * 2;
        let energy_plant = match hex.cell {
            Plant(p) => match p.mass.count() {
                0 => 16,
                n => n,
            },
            _ => 0,
        };

        Some(CellView {
            cell_type,
            energy: Some(energy_plant.saturating_add(energy_vapour)),
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

impl RainfallSim {
    pub fn measure_size(&self) -> f32 {
        self.hexes
            .iter_cells()
            .filter(|h| matches!(h.cell, Plant(_)))
            .count() as f32
    }
    pub fn measure_edges(&self) -> f32 {
        self.hexes.count_edges(|h| matches!(h.cell, Plant(_))) as f32
    }
}
