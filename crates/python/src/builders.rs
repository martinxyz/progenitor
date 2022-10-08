use std::convert::TryInto;

use progenitor::{builders::Builders as BuildersImpl, Simulation};
use pyo3::prelude::*;

#[pyclass]
pub(crate) struct Builders {
    inner: BuildersImpl,
    // #[pyo3(get)]
    // size: u32,
    // data: [tile::CellContent; 3],
    // data: Array2<Cell>,
}

#[pymethods]
impl Builders {
    #[new]
    fn new(weights: Vec<f32>) -> Self {
        assert_eq!(BuildersImpl::PARAM_COUNT, weights.len());
        Self {
            inner: BuildersImpl::new_with_params(
                &weights.try_into().expect("param_count should match"),
            ),
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

    #[classattr]
    fn param_count() -> usize {
        BuildersImpl::PARAM_COUNT
    }

    fn print_stats(&self) {
        self.inner.print_stats()
    }
}

#[pymodule]
fn builders(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Builders>()?;
    Ok(())
}
