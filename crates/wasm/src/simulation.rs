use progenitor::{
    coords::{self, Rectangle},
    HexgridView, Simulation, SIZE,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn get_size() -> u32 {
    SIZE
}

trait HexSim: Simulation + HexgridView {}
impl<T: Simulation + HexgridView> HexSim for T {}

#[wasm_bindgen(js_name = Simulation)]
pub struct JsSimulation(Box<dyn HexSim>);

impl<T> From<T> for JsSimulation
where
    T: HexSim + 'static,
{
    fn from(sim: T) -> JsSimulation {
        JsSimulation(Box::new(sim))
    }
}

#[wasm_bindgen(js_class = Simulation)]
impl JsSimulation {
    pub fn cell_info(&self, col: i32, row: i32) -> JsValue {
        let pos = coords::Offset { col, row };
        let cell = self.0.cell_view(pos.into());
        serde_wasm_bindgen::to_value(&cell).unwrap()
    }
    pub fn cell_text(&self, col: i32, row: i32) -> String {
        let pos = coords::Offset { col, row };
        self.0
            .cell_text(pos.into())
            .unwrap_or_else(|| "(invalid location)".into())
    }

    pub fn steps(&mut self, count: usize) {
        self.0.steps(count);
    }

    pub fn data(&mut self, viewport: &JsViewport, channel: u8) -> Vec<u8> {
        let viewport = {
            let v = viewport;
            Rectangle {
                pos: coords::Offset {
                    col: v.col,
                    row: v.row,
                }
                .into(),
                width: v.width,
                height: v.height,
            }
        };

        self.0
            .cells_rectangle(viewport)
            .iter()
            .map(|cell| match channel {
                0 => cell.cell_type,
                1 => cell.energy.map(|e| e.clamp(0, 254)).unwrap_or(255),
                2 => match cell.direction {
                    Some(dir) => dir as u8,
                    None => 255,
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

    pub fn viewport_hint(&self) -> JsViewport {
        // FIXME: Bad API choice? I think the JsViewport will create a memory
        // leak. (If I understand correctly, the Rust wrapper will not get freed
        // when the JsViewport object is garbage-collected on the JS side.)
        self.0.viewport_hint().into()
    }
}

// Silly to wrap this? Use a plain JS object instead? (via serde)
#[wasm_bindgen(js_name = Viewport)]
pub struct JsViewport {
    pub col: i32,
    pub row: i32,
    pub width: i32,
    pub height: i32,
}

impl From<Rectangle> for JsViewport {
    fn from(r: Rectangle) -> Self {
        let pos: coords::Offset = r.pos.into();
        Self {
            col: pos.col,
            row: pos.row,
            width: r.width,
            height: r.height,
        }
    }
}
