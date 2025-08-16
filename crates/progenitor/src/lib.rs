#[macro_use]
extern crate num_derive;

// foundation
pub mod coords;
pub use coords::{Direction, DirectionSet, Neighbourhood};
mod bit_particles;
use bit_particles::BitParticles;
mod hexgrid_view;
mod probabilities;
mod simulation;
mod tiled;
pub use hexgrid_view::{CellView, HexgridView};
pub use simulation::Simulation;

// map storage
mod axial_tile;
mod hexmap;
mod torus_tile;
pub use axial_tile::AxialTile;
pub use torus_tile::{NeighbourIter as TorusNeighbourIter, SIZE, TorusTile, VIEWPORT};

// shared simulation behaviour
mod ca;
mod independent_pairs;

// simulations
pub mod builders;
pub mod falling_sand;
pub mod falling_sand_v2;
pub mod growth;
pub mod pairs;
pub mod rainfall;
pub mod sim1;
pub mod sunburn;
pub mod tumblers;
pub mod turing;
pub mod world1;

pub type SimRng = rand_xoshiro::Xoshiro256PlusPlus;
