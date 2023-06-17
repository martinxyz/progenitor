#![allow(clippy::new_without_default)]
// #![feature(array_windows)]

// foundation
pub mod coords;
pub use coords::{Direction, DirectionSet};
mod hexgrid_view;
mod simulation;
pub use hexgrid_view::{CellView, HexgridView};
pub use simulation::Simulation;

// map storage
mod axial_tile;
mod torus_tile;
pub use axial_tile::AxialTile;
pub use torus_tile::{NeighbourIter as TorusNeighbourIter, TorusTile, SIZE, VIEWPORT};

// shared simulation behaviour
mod ca;

// simulations
pub mod builders;
pub mod falling_sand;
pub mod sim1;
pub mod tumblers;
pub mod turing;
pub mod world1;

pub type SimRng = rand_pcg::Lcg64Xsh32;
// Xoshiro256PlusPlus is worth trying (some day). While less simple, it's faster
// and just as good, and allows jumps (for parallel computations).
