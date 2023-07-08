use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

use crate::ca;
use crate::coords;
use crate::coords::Direction;
use crate::hexmap;
use crate::AxialTile;
use crate::CellView;
use crate::DirectionSet;
use crate::HexgridView;
use crate::Neighbourhood;
use crate::SimRng;
use crate::Simulation;

const RADIUS: i32 = 15;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
enum CellType {
    Border,
    Sun,
    Stone,
    Air,
    Dust,
    // Stone0(Direction),
}

impl CellType {
    fn transparent(self) -> bool {
        match self {
            CellType::Sun | CellType::Air => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
struct Cell {
    kind: CellType,
    photons: DirectionSet,
}

const BORDER: Cell = Cell {
    kind: CellType::Border,
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
            Direction::NorthWest => Direction::NorthEast,
            Direction::NorthEast => Direction::NorthWest,
            Direction::East => Direction::SouthEast,
            Direction::SouthEast => Direction::East,
            Direction::SouthWest => Direction::West,
            Direction::West => Direction::SouthWest,
        })
    }
    fn pairwise_swap_2(self) -> DirectionSet {
        self.transmuted(|dir| match dir {
            Direction::NorthWest => Direction::West,
            Direction::NorthEast => Direction::East,
            Direction::East => Direction::NorthEast,
            Direction::SouthEast => Direction::SouthWest,
            Direction::SouthWest => Direction::SouthEast,
            Direction::West => Direction::NorthWest,
        })
    }
}

impl ca::TransactionalCaRule for Rule {
    type Cell = Cell;

    fn transaction(
        &self,
        source: Cell,
        target: Cell,
        _direction: Direction,
    ) -> Option<ca::TransactionResult<Cell>> {
        let swap = Some(ca::TransactionResult {
            source: Cell {
                kind: target.kind,
                photons: source.photons,
            },
            target: Cell {
                kind: source.kind,
                photons: target.photons,
            },
        });
        #[allow(clippy::collapsible_if)]
        if target.kind == CellType::Air {
            if source.kind == CellType::Dust {
                return swap;
            }
        }
        None
    }

    fn step(&self, nh: Neighbourhood<Cell>, rng: &mut SimRng) -> Cell {
        let kind = nh.center.kind;
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
                CellType::Sun => emit,
                CellType::Air => transmit,
                _ => match rng.gen::<u8>() % 8 {
                    0 | 1 | 2 => diffuse1,
                    3 | 4 | 5 => diffuse2,
                    _ => reflect,
                },
            }
        };
        Cell { kind, photons }
    }
}

impl SunburnWorld {
    pub fn new() -> SunburnWorld {
        let mut rng = Pcg32::from_rng(thread_rng()).unwrap();

        let cells = hexmap::new(RADIUS, BORDER, |location| {
            // let random_heading = Direction::try_from(rng.gen::<u8>() as i32 % 8).ok();
            let kind = if location.dist_from_top() == 0 {
                CellType::Sun
            } else if location.dist_from_border() == 0 {
                CellType::Stone
            } else if location.dist_from_top() < RADIUS / 4 {
                CellType::Air
            } else {
                match location.dist_from_center() {
                    0..=1 => CellType::Dust,
                    2 => CellType::Air,
                    // _ if rng.gen_bool(0.08) => Cell::Stone(random_heading),
                    _ if rng.gen_bool(0.18) => CellType::Stone,
                    _ => CellType::Air,
                }
            };
            Cell {
                kind,
                photons: DirectionSet::none(),
            }
        });
        SunburnWorld { cells, rng }
    }
}

impl Simulation for SunburnWorld {
    fn step(&mut self) {
        let rule = Rule {};
        self.cells = ca::step_axial(&self.cells, BORDER, &rule, &mut self.rng);
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
                CellType::Air => 2,
                CellType::Sun => 5,
                CellType::Dust => 0,
                CellType::Border => return None,
                CellType::Stone => 4,
            },
            direction: None,
            energy: Some(cell.photons.count()),
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
