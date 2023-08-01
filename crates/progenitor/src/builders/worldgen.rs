use super::wall_sim::Cell;
use crate::{hexmap, AxialTile};
use crate::{Direction, Neighbourhood};
use hex2d::Coordinate;
use rand::seq::SliceRandom;
use rand::Rng;

pub const RING_RADIUS: i32 = 23;

pub fn create_world(rng: &mut impl Rng) -> AxialTile<Cell> {
    let mut cells = hexmap::new(RING_RADIUS, Cell::Border, |loc| {
        let c = loc.dist_from_center();
        let wall_prob: f32 = match c % 4 {
            0 => 0.3,
            _ => 0.1,
        };
        if rng.gen_bool(wall_prob as f64) {
            return Cell::Wall;
        }
        if rng.gen_bool(0.05) && loc.dist_from_top() > RING_RADIUS {
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
                    if rng.gen_bool(0.05) {
                        return Cell::Wall;
                    } else if walls == 1 {
                        if rng.gen_bool(0.1) {
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
    let reject_prob_decay = 0.999; // lower values mean closer to center
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
                if !rng.gen_bool(reject_prob) {
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
