// warning probably caused by macro use
#![allow(clippy::borrow_deref_ref)]

use pyo3::prelude::*;

mod builders;
mod tumblers;
mod world;

#[pymodule]
fn progenitor(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<world::World>()?;
    m.add_class::<tumblers::Tumblers>()?;
    m.add_class::<builders::Builders>()?;
    m.add_class::<builders::Params>()?;
    Ok(())
}
