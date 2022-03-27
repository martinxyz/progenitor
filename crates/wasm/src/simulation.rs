use progenitor::{coords, Simulation, SIZE};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn get_size() -> u32 {
    SIZE
}

#[wasm_bindgen(js_name = Simulation)]
pub struct JsSimulation(Box<dyn Simulation>);

impl<T> From<T> for JsSimulation
where
    T: Simulation + 'static,
{
    fn from(sim: T) -> JsSimulation {
        JsSimulation(Box::new(sim))
    }
}

#[wasm_bindgen(js_class = Simulation)]
impl JsSimulation {
    pub fn get_cell_info(&self, col: i32, row: i32) -> JsValue {
        let pos = coords::Offset { col, row };
        let cell = self.0.get_cell_view(pos.into());
        JsValue::from_serde(&cell).unwrap()
    }

    pub fn step(&mut self) {
        self.0.step();
    }

    pub fn get_data(&mut self, channel: u8) -> Vec<u8> {
        self.0
            .get_cells_rectangle()
            .iter()
            .map(|cell| match channel {
                0 => cell.cell_type,
                1 => cell.energy.unwrap_or(0),
                2 => match cell.direction {
                    Some(dir) => dir as u8,
                    None => 0, // better API contract required...
                },
                _ => panic!("invalid channel"),
            })
            .collect()
    }

    pub fn export_snapshot(&self) -> Vec<u8> {
        self.0.save_state()
    }

    pub fn import_snapshot(&mut self, data: &[u8]) {
        self.0.load_state(data);
    }
}