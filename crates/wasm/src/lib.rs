use wasm_bindgen::prelude::*;

mod simulation;
pub use crate::simulation::*;

mod sim1;
pub use crate::sim1::*;

mod sim2;
pub use crate::sim2::*;

#[wasm_bindgen]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn is_debug_build() -> bool {
    cfg!(debug_assertions)
}
