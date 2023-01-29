//use pyo3::wrap_pyfunction;
extern crate apollo_compiler;

use apollo_compiler::{ApolloCompiler, ApolloDiagnostic, FileId as ApolloFileId};
use apollo_compiler::diagnostics::SyntaxError;
// this is private :( use apollo_compiler::validation::validation_db::{validate_executable};
use pyo3::prelude::*;

use crate::apollo_compiler::database::{AstDatabase, HirDatabase, InputDatabase};
use crate::apollo_compiler::validation::ValidationDatabase;
use crate::ast::gql_core::converter::CoreConversionContext;

mod ast;

//use pyo3::types::{PyString,PyUnicode};


#[pyclass]
struct ValidationResult {
    diagnostics: Vec<ApolloDiagnostic>,
}

#[pymethods]
impl ValidationResult {
    fn get_diagnostics(&self) -> PyResult<()> {
        //self.diagnostics.clone();
        Ok(())
    }
}

#[pyclass]
struct QueryCompiler {
    compiler: ApolloCompiler,
    conversion_context: CoreConversionContext,
}

#[pyclass]
#[derive(Clone)]
struct FileId {
    file_id: ApolloFileId,
}

impl FileId {
    fn from_file_id(file_id: ApolloFileId) -> Self {
        Self {
            file_id,
        }
    }
}

#[pymethods]
impl QueryCompiler {
    #[new]
    fn new() -> Self {
        // create and return a new instance of MyRustObject
        let conversion_context = Python::with_gil(|py| {
            CoreConversionContext::new(py)
        });
        Self {
            compiler: ApolloCompiler::new(),
            conversion_context,
        }
    }

    fn set_schema(&mut self, schema: &str) -> PyResult<()> {
        self.compiler.add_type_system(schema, "schema.graphql");
        Ok(())
    }


    fn add_executable(&mut self, contents: &str) -> PyResult<FileId> {
        // the path is optional and just used in diagnostics, it doesn't need to be unique
        let file_id = self.compiler.add_executable(contents, "");
        // return the file id as a u64
        Ok(FileId::from_file_id(file_id))
    }

    fn add_validate(&mut self, contents: &str) -> PyResult<bool> {
        let file_id = self.add_executable(contents).unwrap();
        let validation_result = self.validate_file(file_id);
        Ok(validation_result.unwrap())
    }

    fn validate_file(&mut self, file_id: FileId) -> PyResult<bool> {
        //self.compiler.db.storage.
        //self.compiler.db.
        //let diagnostics = validate_executable(self.compiler.db,file_id);
        let apollo_file_id = file_id.file_id;
        //let diagnostics = self.compiler.db.validate_operation_definitions(file_id);
        // extracted from ast.rs/check_syntax - we ONLY want to check the syntax for a single ast, not traverse the entire AST
        // - there is no cached method on the db available yet
        let mut diagnostics = self.compiler.db.ast(apollo_file_id)
            .errors()
            .into_iter()
            .map(|err| {
                ApolloDiagnostic::SyntaxError(SyntaxError {
                    src: self.compiler.db.source_code(apollo_file_id),
                    span: (err.index(), err.data().len()).into(), // (offset, length of error token)
                    message: err.message().into(),
                })
            }).collect::<Vec<ApolloDiagnostic>>();

        //diagnostics.extend(self.compiler.db.validate_operation_definitions(apollo_file_id));
        //diagnostics.extend(self.compiler.db.validate_unused_variable(apollo_file_id));

        let errors_count = diagnostics.iter().filter(|d| d.is_error()).count();

        Ok(errors_count == 0)
    }

    fn validate(&mut self) -> PyResult<bool> {
        // implement the validate function here
        // fixme validate is not sufficient, the entire database is validated here. We only want our file to be validated
        let diagnostics = self.compiler.validate();

        let errors_count = diagnostics.iter().filter(|d| d.is_error()).count();

        Ok(errors_count == 0)
    }

    fn gql_core_ast(&mut self, py: Python<'_>, file_id: FileId) -> PyResult<PyObject> {
        // let ast = self.compiler.db.ast(file_id.file_id);
            let gql_core_ast =
                self.conversion_context.convert_core_to_core_ast(py, &self.compiler, file_id.file_id);
            Ok(gql_core_ast?)
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