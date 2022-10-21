use std::convert::TryInto;

use progenitor::{
    builders::{Builders as BuildersImpl, Hyperparams},
    Simulation,
};
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
    fn new(weights: Vec<f32>, init_fac: f32, bias_fac: f32) -> Builders {
        assert_eq!(BuildersImpl::PARAM_COUNT, weights.len());
        Self {
            inner: BuildersImpl::new_with_params(
                &weights.try_into().expect("param_count should match"),
                Hyperparams { init_fac, bias_fac },
            ),
        }
    }

    fn steps(&mut self, count: usize) {
        self.inner.steps(count);
    }

    fn avg_visited(&self) -> f32 {
        self.inner.avg_visited()
    }

    fn encounters(&self) -> f32 {
        self.inner.encounters() as f32
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