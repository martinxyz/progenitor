#![allow(clippy::new_without_default)]

// foundation
pub mod coords;
pub use coords::{Direction, DirectionSet};
mod simulation;
pub use simulation::{CellView, Simulation};

// map storage
mod axial_tile;
mod torus_tile;
pub use axial_tile::AxialTile;
pub use torus_tile::{NeighbourIter as TorusNeighbourIter, TorusTile, SIZE};

// simulations
pub mod blobs;
pub mod sim1;
pub mod sim2;
pub mod tumblers;
pub mod turing;
pub mod world1;
