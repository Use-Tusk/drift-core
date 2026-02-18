use pyo3::prelude::*;

pub fn map_core_err(e: drift_rust_core::CoreError) -> PyErr {
    pyo3::exceptions::PyValueError::new_err(e.to_string())
}
