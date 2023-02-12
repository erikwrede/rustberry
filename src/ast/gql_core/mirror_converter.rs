use std::sync::Arc;

use apollo_compiler::{ApolloCompiler, ApolloDiagnostic, FileId};
use apollo_compiler::database::{AstDatabase, HirDatabase, InputDatabase};
use apollo_compiler::database::hir::{FragmentDefinition, OperationDefinition};
use apollo_compiler::hir::{ByName, OperationType};
use apollo_parser::ast::Selection::Field;
use pyo3::{PyAny, Python};
use pyo3::callback::IntoPyCallbackOutput;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString};

use crate::ast::gql_core::reduced_core_mirror::*;

pub fn convert_to_core_ast(compiler: &ApolloCompiler, file_id: FileId) {
    let ast: Arc<Vec<Arc<OperationDefinition>>> = compiler.db.operations(file_id);
    let ast: ByName<FragmentDefinition> = compiler.db.fragments(file_id);
}

struct CoreOperationType {
    Query: Py<PyAny>,
    Mutation: Py<PyAny>,
    Subscription: Py<PyAny>,
}

impl CoreOperationType {
    fn new(PyOperationType: &PyAny) -> CoreOperationType {
        let query = PyOperationType.getattr("QUERY").unwrap();
        let mutation = PyOperationType.getattr("MUTATION").unwrap();
        let subscription = PyOperationType.getattr("SUBSCRIPTION").unwrap();

        CoreOperationType {
            Query: query.into(),
            Mutation: mutation.into(),
            Subscription: subscription.into(),
        }
    }

    fn get_operation_type(&self, operation_type: OperationType) -> Py<PyAny> {
        let operation_type = match operation_type {
            apollo_compiler::hir::OperationType::Query => self.Query.to_owned(),
            apollo_compiler::hir::OperationType::Mutation => self.Mutation.to_owned(),
            apollo_compiler::hir::OperationType::Subscription => self.Subscription.to_owned(),
        };
        operation_type
    }
}

pub struct MirrorConversionContext {
    //graphql_core_ast: Py<PyModule>,
    operation_type: CoreOperationType,
    operation_definition: Py<PyAny>,
    selection_set_node: Py<PyAny>,
    field_node: Py<PyAny>,
    document_node: Py<PyAny>,
    name_node: Py<PyAny>,
}

impl MirrorConversionContext {
    pub fn new(py: Python) -> Self {
        let graphql_core_ast = PyModule::import(py, "graphql.language.ast").unwrap();
        let PyDocumentNode = graphql_core_ast.getattr("DocumentNode").unwrap();
        let PyOperationType = graphql_core_ast.getattr("OperationType").unwrap();
        let PyOperationDefinitionNode = graphql_core_ast.getattr("OperationDefinitionNode").unwrap();
        let PySelectionSetNode = graphql_core_ast.getattr("SelectionSetNode").unwrap();
        let PyFieldNode = graphql_core_ast.getattr("FieldNode").unwrap();
        let PyNameNode = graphql_core_ast.getattr("NameNode").unwrap();

        Self {
            operation_type: CoreOperationType::new(PyOperationType),
            operation_definition: PyOperationDefinitionNode.into(),
            selection_set_node: PySelectionSetNode.into(),
            field_node: PyFieldNode.into(),
            document_node: PyDocumentNode.into(),
            name_node: PyNameNode.into(),
        }
    }

    fn get_name_nome(&self, py: Python, name: &str) -> NameNode {
        NameNode {
            value: name.to_string(),
        }
    }

    fn convert_field_to_core_field(&self, py: Python, field: &apollo_compiler::hir::Field) -> FieldNode {
        //println!("Converting field to core field...");

        let selection_set = if field.selection_set().selection().len() > 0 {
            //println!("Field has selection set");
            let selection_set = self.convert_selection_set_to_core_selection_set(py, field.selection_set());
            Some(selection_set)
            //println!("Selection set converted!");
        } else {
            None
        };



        //println!("Alias");
        let alias = if let Some(field_alias) = field.alias() {
            Some(self.get_name_nome(py, field_alias.0.as_str()))
        } else {
            None
        };

        //println!("Name");
        let name = self.get_name_nome(py, field.name());

        //println!("Initing lists");

        FieldNode {
            alias: alias,
            name: name,
            arguments: vec![],
            directives: vec![],
            selection_set: selection_set,
        }
    }

    fn convert_selection_set_to_core_selection_set(&self, py: Python, selection_set: &apollo_compiler::hir::SelectionSet) -> SelectionSetNode {
        //println!("Converting selection set...");
        let selection_set_kwargs = PyDict::new(py);
        // FIXME do we NEED to use PyTuple here?
        let selections: Vec<FieldNode> = selection_set.selection().iter().filter_map(|selection| {
            let value = match selection {
                apollo_compiler::hir::Selection::Field(field) => Some(self.convert_field_to_core_field(py, field)),
                apollo_compiler::hir::Selection::FragmentSpread(fragment_spread) => None,//self.convert_fragment_spread_to_core_fragment_spread(py, fragment_spread),
                apollo_compiler::hir::Selection::InlineFragment(inline_fragment) => None,// self.convert_inline_fragment_to_core_inline_fragment(py, inline_fragment),
            };
            value
        }).collect();

        SelectionSetNode {
            selections: selections,
        }
    }

    pub fn convert_core_to_core_ast(self: &Self, py: Python, compiler: &ApolloCompiler, file_id: FileId) -> DocumentNode {
        let operations: Arc<Vec<Arc<OperationDefinition>>> = compiler.db.operations(file_id);
        let fragments: ByName<FragmentDefinition> = compiler.db.fragments(file_id);

        let core_operations: Vec<OperationDefinitionNode> = operations.iter().map(|operation| {
            let operation_kwargs = PyDict::new(py);

            let operation_name = if let Some(operation_name) = operation.name() {
                Some(self.get_name_nome(py, operation_name))
            } else {
                None
            };


            let operation_type = self.operation_type.get_operation_type(operation.operation_ty());

            //println!("Operation type resolved!");
            /*
            directives: PyTuple["DirectiveNode", ...]
            variable_definitions: Tuple["VariableDefinitionNode", ...]
            selection_set: "SelectionSetNode" */

            let directives = operation.directives();
            let variable_definitions = operation.variables();
            let selection_set = operation.selection_set();

            //println!("Selection sett, directives, variables done!");

            let selection_set = self.convert_selection_set_to_core_selection_set(py, selection_set);

            OperationDefinitionNode {
                operation: operation_type,
                name: operation_name,
                variable_definitions: Vec::new(),
                directives: Vec::new(),
                selection_set: selection_set,
            }
        }).collect();

        DocumentNode {
            definitions: core_operations,
        }
    }
}