use apollo_compiler::execution::{GraphQLError as ApolloGraphQLError, ResponseDataPathElement};
use pyo3::{Py, PyAny, pyclass, pymethods, PyObject, PyResult, Python, ToPyObject};
use pyo3::types::{PyInt, PyString};

use crate::ast::gql_core::imports::import_graphql_error;
use crate::ast::gql_core::location::SourceLocation;

#[pyclass]
pub struct GraphQLError {
    pub compiler_error: ApolloGraphQLError,
}

#[pymethods]
impl GraphQLError {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = import_graphql_error(py)?;
        Ok(field_node.into())
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
    pub fn path(&self, py: Python<'_>) -> Vec<PyObject> {
        self.compiler_error
            .path
            .iter()
            .map(|p| match p {
                ResponseDataPathElement::Field(name) => {
                    PyString::new_bound(py, name.as_str()).into()
                }
                ResponseDataPathElement::ListIndex(index) => index.to_object(py),
            })
            .collect()
    }

    #[getter]
    pub fn extensions(&self) -> Option<&PyAny> {
        // raise not supported error
        panic!("Not supported yet")
    }
}
