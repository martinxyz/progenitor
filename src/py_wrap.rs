// use ndarray::{ArrayD, ArrayViewD, ArrayViewMutD};
// use numpy::{IntoPyArray, PyArrayDyn};

//use pyo3::prelude::{pymodinit, Py, PyModule, PyResult, Python};
use pyo3::prelude::*;
// use pyo3::types::{
//     PyBytes
// };
use ndarray::prelude::*;
use numpy::{PyArray2, ToPyArray};

use pyo3::buffer::PyBuffer;
use pyo3::exceptions;
// use ndarray::{self, array, };
// use pyo3::class::PyBufferProtocol;
// use pyo3::wrap_pyfunction;
use crate::tile;  // XXX there is some more elegant way for that somewhere?

#[pyclass]
pub struct Cells {
    #[pyo3(get)]
    size: i32,
    // data: [tile::CellContent; 3],
    data: Array2<tile::CellState>,
}

#[pymethods]
impl Cells {
    #[new]
    fn new() -> Self {
        // let init = tile::CellContent::Empty;
        let init = tile::CellState::Cell{
            cell_type: 3,
            child_count: 34,
            particle: false
        };

        const N: usize = tile::SIZE as usize;
        Self {
            size: 42,
            data: Array::from_elem((N, N), init)
        }
    }

    fn get_particles(&self, py: Python, _x: i32, _y: i32, _w: i32, _h: i32) -> Py<PyArray2<bool>> {
        use tile::CellState::{Border, Empty, Cell};
        let res = self.data.mapv(|item| match item {
            Border | Empty => false,
            Cell{particle, ..} => particle,
        });
        res.to_pyarray(py).to_owned()
    }

    // Actually, PyArray2<bool> seems broken in this context, it results in a
    // type mismatch even when I pass bool from Python.
    fn set_particles(&mut self, data: &PyArray2<u8>, _x: i32, _y: i32) -> PyResult<()> {
        // PyArray2<u8>, (tile_size, tile_size)>
        use tile::CellState::*;  // {Border, Empty, Cell};
        let arr = data.as_array();
        let mut bad = false;
        // println!("data shape: {:?}", arr.dim());
        // todo: use "let if" instead of match
        self.data.zip_mut_with(&arr, |cc, &new_state| match cc {
            Cell{ref mut particle, ..} => {
                *particle = match new_state {0 => false, _ => true};
            },
            _ => {
                bad = true;
            }
        });
        if bad {
            exceptions::TypeError::into("Cannot set particle=true on Empty or Border cells.")
        } else {
            Ok(())
        }
    }


    // fn apply_lut_filter(&mut self, v: Vec<u8>) {
    //     println!("Foo! {}", v.len());
    // }

    // fn test_buffer_protocol(&mut self, buf: &PyBuffer) {
    fn test_buffer_protocol(&self, v: &PyAny) -> PyResult<()> {
        // this works, but we already have the GIL so it looks redundant?
        let gil = Python::acquire_gil();
        let py = gil.python();
        let buf = PyBuffer::get(py, v)?;
        println!("Buffer protocol shape: {:?}", buf.shape());
        Ok(())
    }

    // fn apply_lut_filter(&mut self, buf: &Vec<i32>) {
    //     // println!("Foo!");
    //     println!("Foo! Shape: {:?}", buf.len());
    // }

}

// struct TorusTileStore {  /// (CellsTorus?)
//   CellContent& at(int x, int y) {
//     bool odd = (static_cast<unsigned>(y) / Tile::size) % 2 != 0;
//     if (odd) x += Tile::size/2;
//     // note: can be optimized by storing pointer to inside of padding
//     x = (static_cast<unsigned>(x) % Tile::size) + tile_.padding;
//     y = (static_cast<unsigned>(y) % Tile::size) + tile_.padding;
//     return tile_.data[y*tile_.stride_y + x];
//   }
//   std::vector<Tile*> iter_tiles() {
//     return std::vector<Tile*>{&tile_};
//   }
//   void update_borders() {
//     constexpr int size = Tile::size;
//     constexpr int padding = Tile::padding;
//     constexpr int h = Tile::stride_y;
//     for (int i=0; i<size + 2*padding; i++) {
//       for (int k=0; k<padding; k++) {
//         tile_.data[i*h + k] = tile_.data[i*h + k+size];
//         tile_.data[i*h + padding+size+k] = tile_.data[i*h + padding+k];
//         tile_.data[(k)*h + i] = tile_.data[(k+size)*h + (i+size/2)%size];
//         tile_.data[(padding+size+k)*h + i] = tile_.data[(padding+k)*h + (i+size/2)%size];
//       }
//     }
//   }
//  private:
//   Tile tile_{};
// };
