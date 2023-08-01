// those are a bit too pedantic IMO (in some cases, fixing them would hurt readability)
#![allow(
    clippy::new_without_default,
    clippy::match_like_matches_macro,
    clippy::manual_range_patterns,
    clippy::collapsible_if,
    clippy::let_and_return,
    clippy::identity_op  // I'll multiply by one whenever I like!
)]

// foundation
pub mod coords;
pub use coords::{Direction, DirectionSet, Neighbourhood};
mod hexgrid_view;
mod simulation;
pub use hexgrid_view::{CellView, HexgridView};
pub use simulation::Simulation;

// map storage
mod axial_tile;
mod hexmap;
mod torus_tile;
pub use axial_tile::AxialTile;
pub use torus_tile::{NeighbourIter as TorusNeighbourIter, TorusTile, SIZE, VIEWPORT};

// shared simulation behaviour
mod ca;

// simulations
pub mod builders;
pub mod falling_sand;
pub mod sim1;
pub mod sunburn;
pub mod tumblers;
pub mod turing;
pub mod world1;

pub type SimRng = rand_pcg::Lcg64Xsh32;
// Xoshiro256PlusPlus is worth trying (some day). While less simple, it's faster
// and just as good, and allows jumps (for parallel computations).
