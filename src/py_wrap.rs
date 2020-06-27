// use pyo3::wrap_pyfunction;
use pyo3::buffer::PyBuffer;
use pyo3::prelude::*;
// use pyo3::types::PyBytes;
use ndarray::prelude::*;
// use ndarray::{ArrayD, ArrayViewD, ArrayViewMutD};
// use numpy::convert;
use numpy::{PyArray1, ToPyArray};
// use numpy::{IntoPyArray, PyArray2, PyArrayDyn};

use crate::tile;
// use cell::Cell;

#[pyclass]
pub(crate) struct World {
    inner: crate::World,

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
            inner: crate::World::new(),
            size: tile::SIZE,
        }
    }

    fn get_particles(&self, py: Python, _x: i32, _y: i32, _w: i32, _h: i32) -> Py<PyArray1<bool>> {
        // data: Array::from_elem((N, N), init)
        // use Cell::{Border, Empty, Cell};
        let cells = &self.inner.cells;
        cells
            .iter_cells()
            .map(|cell| cell.get_particle())
            .collect::<Array1<_>>()
            .to_pyarray(py)
            .to_owned()
    }

    // fn test_buffer_protocol(&mut self, buf: &PyBuffer) {
    fn test_buffer_protocol(&self, py: Python, v: &PyAny) -> PyResult<()> {
        let buf = PyBuffer::get(py, v)?;
        println!("Buffer protocol shape: {:?}", buf.shape());
        Ok(())
    }

    // fn apply_lut_filter(&mut self, v: Vec<u8>) {
    //     println!("Foo! {}", v.len());
    // }
    // fn apply_lut_filter(&mut self, buf: &Vec<i32>) {
    //     // println!("Foo!");
    //     println!("Foo! Shape: {:?}", buf.len());
    // }
}

#[pymodule]
fn progenitor(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<World>()?;
    Ok(())
}
