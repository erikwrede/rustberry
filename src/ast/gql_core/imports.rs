use pyo3::{PyObject, PyResult, Python};
use pyo3::prelude::PyAnyMethods;

pub fn import_field_node(py: Python<'_>) -> PyResult<PyObject> {
    let field_node = py
        .import("graphql.language.ast")?
        .getattr("FieldNode")?;
    Ok(field_node.into())
}

pub fn import_graphql_error(py: Python<'_>) -> PyResult<PyObject> {
    let graphql_error = py
        .import("graphql.error.graphql_error")?
        .getattr("GraphQLError")?;
    Ok(graphql_error.into())
}

pub fn import_source_location(py: Python<'_>) -> PyResult<PyObject> {
    let location = py
        .import("graphql.language.location")?
        .getattr("SourceLocation")?;
    Ok(location.into())
}
