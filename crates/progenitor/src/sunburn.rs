use rand::thread_rng;
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

use crate::ca;
use crate::coords;
use crate::coords::Direction;
use crate::coords::Direction::*;
use crate::hexmap;
use crate::AxialTile;
use crate::CellView;
use crate::DirectionSet;
use crate::HexgridView;
use crate::Neighbourhood;
use crate::SimRng;
use crate::Simulation;
use crate::independent_pairs;

const RADIUS: i32 = 15;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
enum CellType {
    Border,
    Sun,
    Stone,
    Air,
    Blob,
}
use CellType::*;

impl CellType {
    fn transparent(self) -> bool {
        match self {
            Sun | Air => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
struct Cell {
    kind: CellType,
    energy: u8,
    photons: DirectionSet,
}

const BORDER: Cell = Cell {
    kind: Border,
    energy: 0,
    photons: DirectionSet::none(),
};

#[derive(Serialize, Deserialize)]
pub struct SunburnWorld {
    cells: AxialTile<Cell>,
    rng: SimRng,
    // visited: AxialTile<Option<bool>>,
}

struct Rule;

impl DirectionSet {
    fn pairwise_swap_1(self) -> DirectionSet {
        self.transmuted(|dir| match dir {
            NorthWest => NorthEast,
            NorthEast => NorthWest,
            East => SouthEast,
            SouthEast => East,
            SouthWest => West,
            West => SouthWest,
        })
    }
    fn pairwise_swap_2(self) -> DirectionSet {
        self.transmuted(|dir| match dir {
            NorthWest => West,
            NorthEast => East,
            East => NorthEast,
            SouthEast => SouthWest,
            SouthWest => SouthEast,
            West => NorthWest,
        })
    }
}

impl ca::TransactionalCaRule for Rule {
    type Cell = Cell;

    fn transaction(
        &self,
        _source: Cell,
        _target: Cell,
        _direction: Direction,
    ) -> Option<ca::TransactionResult<Cell>> {
        None
    }

    fn step(&self, nh: Neighbourhood<Cell>, rng: &mut SimRng) -> Cell {
        if nh.center == BORDER {
            return BORDER;
        }
        let kind = nh.center.kind;
        let energy = nh.center.energy;
        let photons = {
            let incoming = DirectionSet::matching(|dir| {
                nh[dir].photons.contains(-dir)
                // delete photons that cross two walls
                // nh[dir].photons.contains(-dir) && (kind.transparent() || nh[dir].kind.transparent())
            });
            // absorption
            let incoming = {
                let tmp: u8 = if kind.transparent() {
                    rng.gen::<u8>()
                } else {
                    rng.gen::<u8>() % 8
                };
                if let Ok(dir) = Direction::try_from(tmp as i32) {
                    incoming.with(dir, false)
                } else {
                    incoming
                }
            };
            let reflect = incoming;
            let transmit = reflect.mirrored();
            let diffuse1 = reflect.pairwise_swap_1();
            let diffuse2 = reflect.pairwise_swap_2();
            // let bend1 = transmit.pairwise_swap_2();
            // let bend2 = transmit.pairwise_swap_2();
            let emit = DirectionSet::all();

            match kind {
                Sun => emit,
                Air => transmit,
                _ => match rng.gen::<u8>() % 8 {
                    0 | 1 | 2 => diffuse1,
                    3 | 4 | 5 => diffuse2,
                    _ => reflect,
                },
            }
        };

        let mut kind = kind;
        let mut energy = energy;
        let mut photons = photons;
        if kind == Blob {
            energy = (energy as u16 + photons.count() as u16 * 2).clamp(0, 200) as u8;
            photons = DirectionSet::none();
            if energy > 0 {
                // energy -= 1;
                energy -= rng.gen_range(0..=1);
            } else {
                kind = Air;
            }
        }

        Cell { kind, energy, photons }
    }
}

impl SunburnWorld {
    pub fn new() -> SunburnWorld {
        let mut rng = Pcg32::from_rng(thread_rng()).unwrap();

        let cells = hexmap::new(RADIUS, BORDER, |location| {
            let kind = if location.dist_from_top() == 0 {
                Sun
            } else if location.dist_from_border() == 0 {
                Stone
            } else if location.dist_from_top() < RADIUS / 4 {
                Air
            } else {
                match location.dist_from_center() {
                    0..=1 => Blob,
                    2 => Air,
                    _ if rng.gen_bool(0.18) => Stone,
                    _ => Air,
                }
            };
            Cell {
                kind,
                energy: match kind {
                    Blob => rng.gen_range(200..=250),
                    _ => 0
                },
                photons: DirectionSet::none(),
            }
        });
        SunburnWorld { cells, rng }
    }
}

struct Rule2 {}
impl independent_pairs::PairRule for Rule2 {
    type Cell = Cell;

    fn pair_rule(&self,
                 source: Neighbourhood<Self::Cell>,
                 target: Neighbourhood<Self::Cell>,
                 direction: Direction,
                 ) -> (Self::Cell, Self::Cell) {
        let noop = (source.center, target.center);
        let swap = (target.center, source.center);

        match (source.center.kind, target.center.kind) {
            (Air, Blob) => swap,
            (Blob, Air) => {
                let threshold = match direction {
                    NorthWest | NorthEast => 70,
                    East | West => 40,
                    SouthEast | SouthWest => 30,
                };
                if source.center.energy > threshold {
                    let transfer = source.center.energy / 4 * 3;
                    (Cell {
                        energy: source.center.energy - transfer,
                        ..source.center
                    },
                     Cell {
                         kind: source.center.kind,
                         energy: transfer,
                         photons: target.center.photons,
                     })
                } else {
                    noop
                }
            },
            _ => noop
        }
    }
}

impl Simulation for SunburnWorld {
    fn step(&mut self) {
        let rule = Rule {};
        self.cells = ca::step_axial(&self.cells, BORDER, &rule, &mut self.rng);

        let pair_rule = Rule2 {};
        self.cells = independent_pairs::step_axial(&self.cells, BORDER, &pair_rule, &mut self.rng)
    }

    fn save_state(&self) -> Vec<u8> {
        // can we have a default-implementation for Simulation: Serialize + Deserialize
        bincode::serialize(&self).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        *self = bincode::deserialize_from(data).unwrap();
    }
}

impl HexgridView for SunburnWorld {
    fn cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        let cell = self.cells.cell(pos)?;
        Some(CellView {
            cell_type: match cell.kind {
                Air => 2,
                Sun => 5,
                Blob => 3,
                Border => return None,
                Stone => 4,
            },
            direction: None,
            energy: match cell.kind {
                Blob => None,
                _ => Some(cell.photons.count() * 4),
            },
            ..Default::default()
        })
    }
    fn cell_text(&self, pos: coords::Cube) -> Option<String> {
        let cell = self.cells.cell(pos);
        match cell {
            None | Some(BORDER) => None,
            Some(cell) => Some(format!("{cell:?}")),
        }
    }

    fn viewport_hint(&self) -> coords::Rectangle {
        hexmap::viewport(RADIUS)
    }
}

impl Default for SunburnWorld {
    fn default() -> Self {
        Self::new()
    }
}
