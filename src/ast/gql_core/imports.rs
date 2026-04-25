//! Cached lookups of `graphql-core` Python classes.
//!
//! Each helper imports the requested class on first use and stores it in a
//! [`OnceLock`] so subsequent calls are a cheap reference clone.

use std::sync::OnceLock;

use pyo3::prelude::*;
use pyo3::types::PyType;

fn cached<'py>(
    py: Python<'py>,
    slot: &'static OnceLock<Py<PyType>>,
    module: &str,
    name: &str,
) -> PyResult<Py<PyType>> {
    if let Some(cls) = slot.get() {
        return Ok(cls.clone_ref(py));
    }
    let cls: Py<PyType> = py
        .import(module)?
        .getattr(name)?
        .downcast_into::<PyType>()
        .map_err(PyErr::from)?
        .unbind();
    let _ = slot.set(cls.clone_ref(py));
    Ok(cls)
}

pub fn import_field_node(py: Python<'_>) -> PyResult<Py<PyType>> {
    static CLS: OnceLock<Py<PyType>> = OnceLock::new();
    cached(py, &CLS, "graphql.language.ast", "FieldNode")
}

pub fn import_graphql_error(py: Python<'_>) -> PyResult<Py<PyType>> {
    static CLS: OnceLock<Py<PyType>> = OnceLock::new();
    cached(py, &CLS, "graphql.error.graphql_error", "GraphQLError")
}

pub fn import_source_location(py: Python<'_>) -> PyResult<Py<PyType>> {
    static CLS: OnceLock<Py<PyType>> = OnceLock::new();
    cached(py, &CLS, "graphql.language.location", "SourceLocation")
}
