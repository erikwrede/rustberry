//! Python-facing GraphQL error type wrapping an apollo-compiler diagnostic.

use apollo_compiler::response::{GraphQLError as ApolloGraphQLError, ResponseDataPathSegment};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString, PyType};

type PyObject = Py<PyAny>;

use crate::ast::gql_core::imports::import_graphql_error;
use crate::ast::gql_core::location::SourceLocation;

/// Python equivalent of `graphql-core`'s `GraphQLError`, populated from an
/// apollo-compiler diagnostic. Frozen because all data is captured at
/// construction.
#[pyclass(frozen)]
pub struct GraphQLError {
    pub compiler_error: ApolloGraphQLError,
}

#[pymethods]
impl GraphQLError {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        import_graphql_error(py)
    }

    #[getter]
    pub fn message(&self) -> &str {
        self.compiler_error.message.as_str()
    }

    #[getter]
    pub fn locations(&self) -> Vec<SourceLocation> {
        self.compiler_error
            .locations
            .iter()
            .map(|l| SourceLocation {
                line: l.line,
                column: l.column,
            })
            .collect()
    }

    #[getter]
    pub fn path<'py>(&self, py: Python<'py>) -> PyResult<Vec<PyObject>> {
        self.compiler_error
            .path
            .iter()
            .map(|p| match p {
                ResponseDataPathSegment::Field(name) => {
                    Ok(PyString::new(py, name.as_str()).into_any().unbind())
                }
                ResponseDataPathSegment::ListIndex(index) => {
                    Ok(index.into_pyobject(py)?.into_any().unbind())
                }
            })
            .collect()
    }

    /// Always returns `None` for now: apollo-compiler diagnostics do not
    /// carry GraphQL response `extensions` data, so we expose a stable
    /// `None` rather than panicking.
    #[getter]
    pub fn extensions(&self, py: Python<'_>) -> PyResult<PyObject> {
        // Mirror graphql-core's behaviour: extensions defaults to an empty
        // dict if absent. Returning a fresh empty dict keeps consumers
        // happy without leaking shared state.
        Ok(PyDict::new(py).into_any().unbind())
    }

    /// graphql-core's `GraphQLError.original_error` carries the wrapped
    /// exception when an error is re-raised. Apollo-compiler diagnostics are
    /// not wrappers, so we expose `None` to satisfy AttributeError-prone
    /// helpers (e.g. Strawberry's error formatter).
    #[getter]
    pub fn original_error(&self, py: Python<'_>) -> Py<PyAny> {
        py.None()
    }

    /// graphql-core also exposes `nodes`, `source`, and `positions` on
    /// `GraphQLError`. Apollo-compiler diagnostics don't preserve those, but
    /// downstream code occasionally probes them — return `None` so attribute
    /// access never raises.
    #[getter]
    pub fn nodes(&self, py: Python<'_>) -> Py<PyAny> {
        py.None()
    }

    #[getter]
    pub fn source(&self, py: Python<'_>) -> Py<PyAny> {
        py.None()
    }

    #[getter]
    pub fn positions(&self, py: Python<'_>) -> Py<PyAny> {
        py.None()
    }

    /// String representation matches the underlying message — useful when
    /// converting back to a Python `GraphQLError` exception.
    fn __str__(&self) -> &str {
        self.compiler_error.message.as_str()
    }

    fn __repr__(&self) -> String {
        format!("GraphQLError({:?})", self.compiler_error.message)
    }
}
