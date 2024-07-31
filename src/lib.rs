//! cr_trichome
//!
//! This is the Rust side documentation of the cr_trichome crate.
#![deny(missing_docs)]

use pyo3::prelude::*;

mod cell_properties;
mod custom_domain;
mod run_simulation;

use run_simulation::*;

/// A Python module implemented in Rust.
#[pymodule]
#[pyo3(name="_cr_trichome_rust")]
fn _cr_trichome_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SimulationSettings>()?;
    m.add_function(wrap_pyfunction!(run_sim, m)?)?;
    Ok(())
}
