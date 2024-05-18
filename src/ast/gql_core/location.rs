use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict};

use crate::ast::gql_core::imports::import_source_location;

/// Wrapper for SourceLocation class
#[pyclass]
pub struct SourceLocation {
    #[pyo3(get)]
    pub line: usize,
    #[pyo3(get)]
    pub column: usize,
}

#[pymethods]
impl SourceLocation {
    #[getter(__class__)]
    fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        import_source_location(py)
    }

    // #[getter]
    // fn formatted(&self, py: Python<'_>) -> PyResult<Bound<PyDict>> {
    //     let dict = [("line", self.line), ("column", self.column)].into_py_dict_bound(py);
    //     Ok(dict)
    // }
    //
    // fn eq(&self, other: PyObject, py: Python<'_>) -> PyResult<bool> {
    //     let other = other.extract::<&PyDict>(py)?;
    //     let self_formatted = self.formatted(py)?;
    //     Ok(self_formatted.eq(py, other))
    // }

    // fn ne(&self, other: PyObject, py: Python<'_>) -> PyResult<bool> {
    //     Ok(!self.eq(other, py)?)
    // }
}
