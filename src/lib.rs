//! Python bindings for apollo-rs's GraphQL compiler.
//!
//! Exposes a [`QueryCompiler`] class that parses, validates and converts
//! GraphQL queries into a Python AST compatible with `graphql-core`.

use apollo_compiler::validation::Valid;
use apollo_compiler::{ExecutableDocument, Schema};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::ast::gql_core::converter::CoreConversionContext;
use crate::ast::gql_core::error::GraphQLError;
use crate::ast::gql_core::mirror_converter::MirrorConversionContext;
use crate::ast::gql_core::reduced_core_mirror::DocumentNode;

mod ast;
mod util;

/// Opaque handle to a parsed `apollo-compiler` executable document.
///
/// Returned from [`QueryCompiler::parse`] and accepted by
/// [`QueryCompiler::validate`] / [`QueryCompiler::gql_core_ast`] /
/// [`QueryCompiler::gql_core_ast_mirror`].
#[pyclass]
pub struct Document {
    document: ExecutableDocument,
}

/// Persistent compiler that holds a parsed schema and reusable conversion
/// contexts. Re-use a single instance across many queries to amortise the
/// cost of parsing the schema and importing `graphql-core`'s AST module.
#[pyclass]
pub struct QueryCompiler {
    schema: Valid<Schema>,
    conversion_context: CoreConversionContext,
    mirror_conversion_context: MirrorConversionContext,
}

/// Format an apollo-compiler `WithErrors` value into a single human-readable
/// string suitable for use as the message of a Python exception.
fn format_with_errors<T>(errors: &apollo_compiler::validation::WithErrors<T>) -> String {
    errors
        .errors
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[pymethods]
impl QueryCompiler {
    /// Build a compiler from a SDL document. The schema is parsed but not
    /// validated; we treat user-provided schemas as authoritative
    /// (`Valid::assume_valid`).
    #[new]
    fn new(py: Python<'_>, schema: String) -> PyResult<Self> {
        // Parse without holding the GIL – pure CPU work in apollo-rs.
        let parsed = py
            .detach(|| Schema::parse(schema, "schema.graphql"))
            .map_err(|errors| {
                PyValueError::new_err(format!(
                    "failed to parse schema: {}",
                    format_with_errors(&errors)
                ))
            })?;

        let conversion_context = CoreConversionContext::new(py)?;
        let mirror_conversion_context = MirrorConversionContext::new(py)?;

        Ok(Self {
            schema: Valid::assume_valid(parsed),
            conversion_context,
            mirror_conversion_context,
        })
    }

    /// Parse `document` against the schema. Raises `ValueError` on parse
    /// errors.
    fn parse(&self, py: Python<'_>, document: &str) -> PyResult<Document> {
        let result = py.detach(|| {
            ExecutableDocument::parse(&self.schema, document, "document.graphql")
        });

        match result {
            Ok(doc) => Ok(Document { document: doc }),
            Err(with_errors) => Err(PyValueError::new_err(format_with_errors(&with_errors))),
        }
    }

    /// Parse and validate in one shot. Returns a list of
    /// [`GraphQLError`] – empty on success.
    fn add_validate(&self, py: Python<'_>, document: &str) -> PyResult<Vec<GraphQLError>> {
        let parsed = py.detach(|| {
            ExecutableDocument::parse(&self.schema, document, "document.graphql")
        });

        let parsed = match parsed {
            Ok(doc) => doc,
            Err(with_errors) => return Ok(diagnostics_to_errors(&with_errors.errors)),
        };

        let validation = py.detach(|| parsed.validate(&self.schema));
        match validation {
            Ok(_) => Ok(Vec::new()),
            Err(with_errors) => Ok(diagnostics_to_errors(&with_errors.errors)),
        }
    }

    /// Validate an already-parsed [`Document`]. Returns a list of
    /// [`GraphQLError`] – empty on success.
    fn validate(&self, py: Python<'_>, document: &Document) -> PyResult<Vec<GraphQLError>> {
        // `validate` consumes the document; clone the inner document so we
        // don't move it out of the borrow.
        let doc = document.document.clone();
        let result = py.detach(|| doc.validate(&self.schema));
        match result {
            Ok(_) => Ok(Vec::new()),
            Err(with_errors) => Ok(diagnostics_to_errors(&with_errors.errors)),
        }
    }

    /// Convert `document` into a tree of real `graphql-core` AST instances.
    fn gql_core_ast<'py>(
        &self,
        py: Python<'py>,
        document: &Document,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.conversion_context
            .convert_core_to_core_ast(py, &document.document)
    }

    /// Convert `document` into a lightweight Rust-backed mirror AST.
    fn gql_core_ast_mirror(
        &self,
        py: Python<'_>,
        document: &Document,
    ) -> PyResult<Py<DocumentNode>> {
        let doc_node = self
            .mirror_conversion_context
            .convert_core_to_core_ast(py, &document.document)?;
        Py::new(py, doc_node)
    }
}

/// Convert apollo-compiler diagnostics into Python-facing
/// [`GraphQLError`] values.
fn diagnostics_to_errors(
    diagnostics: &apollo_compiler::validation::DiagnosticList,
) -> Vec<GraphQLError> {
    diagnostics
        .iter()
        .map(|diag| GraphQLError {
            compiler_error: diag.to_json(),
        })
        .collect()
}

#[pymodule]
fn _rustberry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<QueryCompiler>()?;
    m.add_class::<Document>()?;
    m.add_class::<GraphQLError>()?;
    m.add_class::<crate::ast::gql_core::location::SourceLocation>()?;
    Ok(())
}
