
use std::sync::Arc;

use apollo_compiler::{ApolloCompiler, ApolloDiagnostic, FileId};

use apollo_compiler::database::{AstDatabase, InputDatabase, HirDatabase};
use apollo_compiler::database::hir::{OperationDefinition,FragmentDefinition};
use apollo_compiler::hir::ByName;

pub fn convert_to_core_ast(compiler: &ApolloCompiler, file_id: FileId) {
        let ast : Arc<Vec<Arc<OperationDefinition>>> = compiler.db.operations(file_id);
        let ast : ByName<FragmentDefinition> = compiler.db.fragments(file_id);

}