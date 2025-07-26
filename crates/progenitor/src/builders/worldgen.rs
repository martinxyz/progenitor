use super::wall_sim::Cell;
use crate::{AxialTile, hexmap};
use crate::{Direction, Neighbourhood};
use hex2d::Coordinate;
use rand::prelude::*;

pub const RING_RADIUS: i32 = 33;

pub fn create_world(rng: &mut impl Rng) -> AxialTile<Cell> {
    let mut cells = hexmap::new(RING_RADIUS, Cell::Border, |loc| {
        let c = loc.dist_from_center();
        let wall_prob: f32 = if c < 10 {
            0.0
        } else {
            match c % 4 {
                0 => 0.9,
                _ => 0.5,
            }
        };
        if rng.random_bool(wall_prob as f64) {
            return Cell::Wall;
        }
        if c > 5 && rng.random_bool(0.05) {
            Cell::Food
        } else {
            Cell::Air
        }
    });

    fn rule(nh: Neighbourhood<Cell>, rng: &mut impl Rng) -> Cell {
        let walls = nh.count_neighbours(|n| n == Cell::Wall);
        let foods = nh.count_neighbours(|n| n == Cell::Food);
        match nh.center {
            Cell::Food => {
                if foods > 0 {
                    return Cell::Air;
                }
            }
            Cell::Air => {
                if walls > 0 {
                    if rng.random_bool(0.05) {
                        return Cell::Wall;
                    } else if walls == 1 {
                        if rng.random_bool(0.1) {
                            return Cell::Wall;
                        }
                    }
                }
            }
            _ => {}
        }
        nh.center
    }

    for _ in 0..6 {
        cells = cells.ca_step(Cell::Border, |nh| rule(nh, rng));
    }

    cells
}

pub fn find_agent_starting_place(rng: &mut impl Rng, cells: &AxialTile<Cell>) -> Coordinate {
    // find a random place that does not look too much like a cage
    let center = hexmap::center(RING_RADIUS);
    let mut reject_prob = 1.0; // approx. ratio of walls nearby
    let reject_prob_decay = 0.9; // lower values mean closer to center
    let mut pos = center;
    let mut heading;
    loop {
        heading = *Direction::all().choose(rng).unwrap();
        let old_pos = pos;
        pos = pos + hex2d::Direction::from(heading);
        match cells.cell(pos) {
            None | Some(Cell::Border) => {
                pos = old_pos;
            }
            Some(Cell::Air) => {
                if !rng.random_bool(reject_prob) {
                    break pos;
                }
                reject_prob *= reject_prob_decay;
            }
            _ => {
                reject_prob *= reject_prob_decay;
                reject_prob += 1. - reject_prob_decay;
            }
        }
    }
}
