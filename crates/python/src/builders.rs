use progenitor::Simulation;
use pyo3::prelude::*;

use progenitor::builders;

#[pyclass]
pub(crate) struct Builders {
    inner: builders::Builders,
    // #[pyo3(get)]
    // size: u32,
    // data: [tile::CellContent; 3],
    // data: Array2<Cell>,
}

#[pymethods]
impl Builders {
    #[new]
    fn new(weights: Vec<f32>) -> Self {
        Self {
            inner: builders::Builders::new_with_agent(builders::builder_agent::categorical_agent(
                &weights,
            )),
        }
    }

    fn steps(&mut self, count: usize) {
        self.inner.steps(count);
    }

    fn avg_visited(&self) -> f32 {
        self.inner.avg_visited()
    }

    fn score(&self) -> f32 {
        self.inner.score()
    }
}

#[pymodule]
fn builders(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Builders>()?;
    Ok(())
}
