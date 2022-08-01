#![allow(clippy::new_without_default)]

pub mod coords;
pub mod tile;
pub use coords::{Direction, DirectionSet};
pub mod blobs;
pub mod sim1;
pub mod sim2;
mod simulation;
pub mod tumblers;
pub mod turing;
pub use simulation::{CellView, Simulation};
pub mod world1;

pub use tile::{Tile, SIZE};
