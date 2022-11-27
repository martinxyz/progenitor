use progenitor::Simulation;
use pyo3::prelude::*;

use progenitor::tumblers;

#[pyclass]
pub(crate) struct Tumblers {
    inner: tumblers::Tumblers,
    // #[pyo3(get)]
    // size: u32,
    // data: [tile::CellContent; 3],
    // data: Array2<Cell>,
}

#[pymethods]
impl Tumblers {
    #[new]
    fn new(prob: f64) -> Self {
        Self {
            inner: tumblers::Tumblers::new(prob),
        }
    }

    fn steps(&mut self, count: usize) {
        self.inner.steps(count);
    }

    fn avg_visited(&self) -> f32 {
        self.inner.avg_visited()
    }
}
