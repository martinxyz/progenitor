use hex2d::Direction;
use hex2d::Spin;
use rand::prelude::IteratorRandom;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use rand::RngCore;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

use crate::coords;
use crate::CellView;
use crate::DirectionSet;
use crate::Simulation;
use crate::TorusTile;

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug)]
struct BlobCell {
    mass: u8,
    ready: DirectionSet,
    valid: DirectionSet,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
enum Cell {
    Empty,
    Wall,
    Blob(BlobCell),
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Empty
    }
}

#[derive(Serialize, Deserialize)]
pub struct Blobs {
    cells: TorusTile<Cell>,
    rng: rand_pcg::Lcg64Xsh32,
}

impl Blobs {
    pub fn new() -> Self {
        let seed = thread_rng().next_u64();
        let rng: rand_pcg::Lcg64Xsh32 = Pcg32::seed_from_u64(seed);
        let mut cells = TorusTile::new(Cell::Empty);

        let size_half = (crate::SIZE / 2) as i32;
        let center: coords::Cube = coords::Offset {
            col: size_half,
            row: size_half,
        }
        .into();

        for pos in center.ring_iter(6, Spin::CCW(Direction::XY)) {
            cells.set_cell(pos, Cell::Wall);
        }

        cells.set_cell(
            center,
            Cell::Blob(BlobCell {
                mass: 3,
                ready: DirectionSet::none(),
                valid: DirectionSet::none(),
            }),
        );

        Blobs { cells, rng }
    }
}

impl From<Cell> for CellView {
    fn from(cell: Cell) -> Self {
        match cell {
            Cell::Empty => CellView {
                cell_type: 0,
                ..Default::default()
            },
            Cell::Wall => CellView {
                cell_type: 1,
                ..Default::default()
            },
            Cell::Blob(blob) => CellView {
                cell_type: 2,
                energy: Some(blob.mass),
                ..Default::default()
            },
        }
    }
}

impl Simulation for Blobs {
    fn step(&mut self) {
        self.cells = self
            .cells
            .iter_radius_1()
            .map(|(c, neigh)| update_cell(c, neigh, &mut self.rng))
            .collect();
    }

    fn get_cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        Some(self.cells.get_cell(pos).into())
    }

    fn save_state(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        *self = bincode::deserialize_from(data).unwrap();
    }

    fn get_cell_text(&self, pos: coords::Cube) -> Option<String> {
        let cell = self.cells.get_cell(pos);
        Some(format!("{:?}", cell))
    }
}

fn update_cell(cell: Cell, neighbours: [(Direction, Cell); 6], rng: &mut impl Rng) -> Cell {
    match cell {
        Cell::Wall => cell,
        Cell::Blob(blob) => {
            let mass_change: i16 = neighbours
                .iter()
                .map(|&(dir, cell2)| match cell2 {
                    Cell::Blob(blob2) => {
                        let ready = blob.ready.contains(dir);
                        let valid = blob.valid.contains(dir);
                        let ready2 = blob2.ready.contains(-dir);
                        let valid2 = blob2.valid.contains(-dir);
                        let incoming: i16 = (ready && valid2).into();
                        let outgoing: i16 = (ready2 && valid).into();
                        incoming - outgoing
                    }
                    _ => 0,
                })
                .sum();

            let mass = (blob.mass as i16 + mass_change).clamp(0, 255) as u8;
            let valid = if mass > 0 {
                DirectionSet::single(*Direction::all().choose(rng).unwrap())
            } else {
                DirectionSet::none()
            };
            let ready = if mass <= 255 - 6 {
                DirectionSet::all()
            } else {
                DirectionSet::none()
            };

            let has_neighbours_with_mass = neighbours
                .iter()
                .any(|&(_, cell2)| matches!(cell2, Cell::Blob(BlobCell { mass, .. }) if mass > 0));

            if blob.mass != 0 || has_neighbours_with_mass {
                Cell::Blob(BlobCell { mass, ready, valid })
            } else {
                Cell::Empty
            }
        }
        Cell::Empty => {
            let candidates = neighbours.iter().filter_map(|&(dir, neigh)| match neigh {
                Cell::Blob(BlobCell { mass, .. }) if (mass > 0) => Some(dir),
                _ => None,
            });
            candidates
                .choose(rng)
                .map(|dir| {
                    Cell::Blob(BlobCell {
                        mass: 0,
                        ready: DirectionSet::single(dir),
                        valid: DirectionSet::none(),
                    })
                })
                .unwrap_or(Cell::Empty)
        }
    }
}
