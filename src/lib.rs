//use pyo3::wrap_pyfunction;
extern crate apollo_compiler;

use apollo_compiler::{ExecutableDocument, Schema};
use apollo_compiler::validation::Valid;
use pyo3::prelude::*;

use crate::ast::gql_core::converter::CoreConversionContext;
use crate::ast::gql_core::mirror_converter::MirrorConversionContext;
use crate::ast::gql_core::reduced_core_mirror::DocumentNode;

mod ast;
mod util;

//use pyo3::types::{PyString,PyUnicode};

#[pyclass]
#[derive(Clone)]
struct Document {
    document: ExecutableDocument,
}

#[pymethods]
impl Document {}

#[pyclass]
struct QueryCompiler {
    schema: Valid<Schema>,
    conversion_context: CoreConversionContext,
    mirror_conversion_context: MirrorConversionContext,
}

#[pymethods]
impl QueryCompiler {
    #[new]
    fn new(schema: String) -> Self {
        let apollo_schema = Schema::parse(schema.clone(), "document.graphql").unwrap();
        let conversion_context = Python::with_gil(|py| CoreConversionContext::new(py));
        let mirror_conversion_context = Python::with_gil(|py| MirrorConversionContext::new(py));

        Self {
            schema: Valid::assume_valid(apollo_schema),
            conversion_context,
            mirror_conversion_context,
        }
    }

    fn parse(&mut self, document: &str) -> PyResult<Document> {
        let result = ExecutableDocument::parse(&self.schema, document, "document.graphql");

        match result {
            Ok(valid_doc) => Ok(Document {
                document: valid_doc,
            }),
            Err(with_errors) => {
                let errors = with_errors.errors;
                panic!("{:?}", errors);
                // let rustberry_errors = errors
                //     .iter()
                //     .map(|e| GraphQLError {
                //         compiler_error: ApolloGraphQLError::new(
                //             e.error.unstable_compat_message().unwrap_or(e.to_string()),
                //             e.error.location(),
                //             e.sources,
                //         ),
                //     });
                // Err(PyErr::new::<ApolloGraphQLError, _>(format!("{:?}", rustberry_errors)))
            }
        }
    }

    fn add_validate(&mut self, document: &str) -> PyResult<bool> {
        let parsed_result = ExecutableDocument::parse(&self.schema, document, "document.graphql");

        if parsed_result.is_err() {
            return Ok(false);
        }

        Ok(parsed_result.unwrap().validate(&self.schema).is_ok())
    }

    fn validate(&mut self, document: Document) -> PyResult<bool> {
        let result = document.document.validate(&self.schema);

        match result {
            Ok(valid_doc) => Ok(true),
            Err(with_errors) => {
                let errors = with_errors.errors;
                // let rustberry_errors = errors
                //     .iter()
                //     .map(|e| GraphQLError {
                //         compiler_error: ApolloGraphQLError::new(
                //             e.error.unstable_compat_message().unwrap_or(e.to_string()),
                //             e.error.location(),
                //             e.sources,
                //         ),
                //     });
                Ok(false)
            }
        }
    }
    fn gql_core_ast(&mut self, py: Python<'_>, document: &Document) -> PyResult<PyObject> {
        // let ast = self.compiler.db.ast(file_id.file_id);
        let gql_core_ast = self
            .conversion_context
            .convert_core_to_core_ast(py, &document.document);
        Ok(gql_core_ast?)
    }

    fn gql_core_ast_mirror(&mut self, py: Python<'_>, document: &Document) -> PyResult<Py<DocumentNode>> {
        // let ast = self.compiler.db.ast(file_id.file_id);
        let gql_core_ast = self
            .mirror_conversion_context
            .convert_core_to_core_ast(py, &document.document);

        Py::new(py, gql_core_ast)
    }
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn _rustberry(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<QueryCompiler>()?;

    Ok(())
}
