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
use crate::HexgridView;
use crate::Neighbourhood;
use crate::SimRng;
use crate::Simulation;

const RADIUS: i32 = 12;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
enum Cell {
    Border,
    Stone,
    Air,
    Dust,
    // Stone0(Direction),
}

#[derive(Serialize, Deserialize)]
pub struct SunburnWorld {
    cells: AxialTile<Cell>,
    rng: SimRng,
    // visited: AxialTile<Option<bool>>,
}

struct Rule;

impl ca::TransactionalCaRule for Rule {
    type Cell = Cell;

    fn transaction(
        &self,
        source: Cell,
        target: Cell,
        _direction: Direction,
    ) -> Option<ca::TransactionResult<Cell>> {
        let swap = Some(ca::TransactionResult {
            source: target,
            target: source,
        });
        if target == Cell::Air {
            if source == Cell::Dust {
                return swap;
            }
        }
        None
    }

    fn step(&self, nh: Neighbourhood<Cell>, _rng: &mut SimRng) -> Cell {
        nh.center
    }
}

impl SunburnWorld {
    pub fn new() -> SunburnWorld {
        let mut rng = Pcg32::from_rng(thread_rng()).unwrap();

        let cells = hexmap::new(RADIUS, Cell::Border, |location| {
            // let random_heading = Direction::try_from(rng.gen::<u8>() as i32 % 8).ok();
            if location.dist_from_top() < RADIUS / 3 {
                return Cell::Air;
            }
            match location.dist_from_center() {
                0..=1 => Cell::Dust,
                2 => Cell::Air,
                // _ if rng.gen_bool(0.08) => Cell::Stone(random_heading),
                _ if rng.gen_bool(0.08) => Cell::Stone,
                _ => Cell::Air,
            }
        });
        SunburnWorld { cells, rng }
    }

    // pub fn seed(&mut self, seed: u64) {
    //     self.rng = Pcg32::seed_from_u64(seed);
    // }
}

impl Simulation for SunburnWorld {
    fn step(&mut self) {
        let rule = Rule {};
        self.cells = ca::step_axial(&self.cells, Cell::Border, &rule, &mut self.rng);
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
            cell_type: match cell {
                Cell::Air => 2,
                // Cell::Grass => 1,
                // Cell::Sand => 2,
                Cell::Dust => 0,
                Cell::Border => return None,
                Cell::Stone => 4,
            },
            direction: None,
            ..Default::default()
        })
    }
    fn cell_text(&self, pos: coords::Cube) -> Option<String> {
        let cell = self.cells.cell(pos);
        match cell {
            None | Some(Cell::Border) => return None,
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
