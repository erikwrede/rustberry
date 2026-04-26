//! Mirror of `graphql.language.location.SourceLocation` for diagnostics.

use pyo3::prelude::*;
use pyo3::types::PyType;

use crate::ast::gql_core::imports::import_source_location;

/// Wrapper that quacks like `graphql-core`'s `SourceLocation`.
#[pyclass(frozen)]
pub struct SourceLocation {
    #[pyo3(get)]
    pub line: usize,
    #[pyo3(get)]
    pub column: usize,
}

#[pymethods]
impl SourceLocation {
    #[getter(__class__)]
    fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        import_source_location(py)
    }
}
