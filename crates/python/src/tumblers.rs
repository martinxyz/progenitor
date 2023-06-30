use progenitor::Simulation;
use pyo3::prelude::*;

use progenitor::tumblers;

#[pyclass]
pub(crate) struct Tumblers {
    inner: tumblers::Tumblers,
    // #[pyo3(get)]
    // size: u32,
}

#[pymethods]
impl Tumblers {
    #[new]
    fn new(seed: u64) -> Self {
        Self {
            inner: tumblers::Tumblers::new_with_seed(seed),
        }
    }

    fn steps(&mut self, count: usize) {
        self.inner.steps(count);
    }

    fn avg_visited(&self) -> f32 {
        self.inner.avg_visited()
    }

    fn count_bots(&self) -> i32 {
        self.inner.count_bots()
    }

    fn loss(&self) -> f32 {
        self.inner.loss()
    }
}
