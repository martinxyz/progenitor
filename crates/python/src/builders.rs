use std::convert::TryInto;

use progenitor::{
    builders::{Builders as BuildersImpl, Hyperparams, Params as ParamsImpl},
    Simulation,
};
use pyo3::{prelude::*, types::PyBytes};

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
    fn new(params: &Params) -> Builders {
        Self {
            inner: BuildersImpl::new_with_params(params.inner.clone()),
        }
    }

    fn steps(&mut self, count: usize) {
        self.inner.steps(count);
    }

    fn avg_visited(&self) -> f32 {
        self.inner.avg_visited()
    }

    fn encounters(&self) -> i32 {
        self.inner.encounters
    }

    fn relative_wall_edges(&self) -> f32 {
        self.inner.relative_wall_edges()
    }

    fn max_depth_reached(&self) -> i32 {
        self.inner.max_depth_reached
    }

    #[classattr]
    fn param_count() -> usize {
        BuildersImpl::PARAM_COUNT
    }

    fn print_stats(&self) {
        self.inner.print_stats()
    }
}

#[pyclass]
pub(crate) struct Params {
    inner: ParamsImpl,
}

#[pymethods]
impl Params {
    #[new]
    fn new(weights: Vec<f32>, init_fac: f32, bias_fac: f32) -> Params {
        assert_eq!(BuildersImpl::PARAM_COUNT, weights.len());
        Self {
            inner: progenitor::builders::Params {
                builder_weights: weights.try_into().expect("param_count should match"),
                builder_hyperparams: Hyperparams { init_fac, bias_fac },
            },
        }
    }

    fn serialize(&self, py: Python) -> PyObject {
        PyBytes::new(py, &bincode::serialize(&self.inner).unwrap()).into()
    }

    #[staticmethod]
    fn deserialize(bytes: &PyBytes) -> Self {
        Self {
            inner: bincode::deserialize(bytes.as_bytes())
                .expect("bytes should deserialize to valid Params"),
        }
    }
}
