// use pyo3::wrap_pyfunction;
// use pyo3::buffer::PyBuffer;
use pyo3::Bound;
use pyo3::prelude::*;
// use pyo3::types::PyBytes;
use ndarray::prelude::*;
// use ndarray::{ArrayD, ArrayViewD, ArrayViewMutD};
// use numpy::convert;
use numpy::{PyArray1, ToPyArray};
// use numpy::{IntoPyArray, PyArray2, PyArrayDyn};

use progenitor::SIZE;
use progenitor::sim1;
// use cell::Cell;

#[pyclass]
pub(crate) struct World {
    inner: sim1::World,

    #[pyo3(get)]
    size: u32,
    // data: [tile::CellContent; 3],
    // data: Array2<Cell>,
}

#[pymethods]
impl World {
    #[new]
    fn new() -> Self {
        Self {
            inner: sim1::World::new(),
            size: SIZE,
        }
    }

    // unused and broken...?
    // fn get_particles<'py>(
    //     &self,
    //     py: Python<'py>,
    //     _x: i32,
    //     _y: i32,
    //     _w: i32,
    //     _h: i32,
    // ) -> Bound<'py, PyArray1<bool>> {
    //     // data: Array::from_elem((N, N), init)
    //     // use Cell::{Border, Empty, Cell};
    //     let cells = &self.inner.cells;
    //     cells
    //         .iter_cells()
    //         .map(|cell| cell.particles.count() > 0)
    //         .collect::<Array1<_>>()
    //         .to_pyarray(py)
    //         .to_owned()
    // }

    // fn test_buffer_protocol(&mut self, buf: &PyBuffer) {
    // fn test_buffer_protocol(&self, v: &PyAny) -> PyResult<()> {
    //     let buf: PyBuffer<u8> = PyBuffer::get(v)?;
    //     println!("Buffer protocol shape: {:?}", buf.shape());
    //     Ok(())
    // }

    // fn apply_lut_filter(&mut self, v: Vec<u8>) {
    //     println!("Foo! {}", v.len());
    // }
    // fn apply_lut_filter(&mut self, buf: &Vec<i32>) {
    //     // println!("Foo!");
    //     println!("Foo! Shape: {:?}", buf.len());
    // }
}
