use apollo_compiler::{ExecutableDocument, Node};
use apollo_compiler::executable::{Field, OperationType, Selection, SelectionSet};
use pyo3::{PyAny, Python};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::ast::gql_core::reduced_core_mirror::*;

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
        match operation_type {
            OperationType::Query => self.Query.to_owned(),
            OperationType::Mutation => self.Mutation.to_owned(),
            OperationType::Subscription => self.Subscription.to_owned(),
        }
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
        let PyOperationDefinitionNode =
            graphql_core_ast.getattr("OperationDefinitionNode").unwrap();
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

    fn get_name_node(&self, py: Python, name: &str) -> NameNode {
        NameNode {
            value: name.to_string(),
        }
    }

    fn convert_field_to_core_field(&self, py: Python, field: &Node<Field>) -> FieldNode {

        let selection_set = field.selection_set.selections.first()
            .map(|_| self.convert_selection_set_to_core_selection_set(py, &field.selection_set));


        let alias = field.alias.as_ref().map(|field_alias| self.get_name_node(py, field_alias.as_str()));

        let name = self.get_name_node(py, field.name.as_str());

        FieldNode {
            alias,
            name: name,
            arguments: vec![],
            directives: vec![],
            selection_set: selection_set,
        }
    }

    fn convert_selection_set_to_core_selection_set(
        &self,
        py: Python,
        selection_set: &SelectionSet,
    ) -> SelectionSetNode {
        //println!("Converting selection set...");
        let selection_set_kwargs = PyDict::new(py);
        // FIXME do we NEED to use PyTuple here?
        let selections: Vec<FieldNode> = selection_set
            .selections
            .iter()
            .filter_map(|selection| {
                let value = match selection {
                    Selection::Field(field) => Some(self.convert_field_to_core_field(py, field)),
                    Selection::FragmentSpread(fragment_spread) => None, //self.convert_fragment_spread_to_core_fragment_spread(py, fragment_spread),
                    Selection::InlineFragment(inline_fragment) => None, // self.convert_inline_fragment_to_core_inline_fragment(py, inline_fragment),
                };
                value
            })
            .collect();

        SelectionSetNode {
            selections: selections,
        }
    }

    pub fn convert_core_to_core_ast(
        self: &Self,
        py: Python,
        document: &ExecutableDocument,
    ) -> DocumentNode {
        let operations = document.all_operations();
        let fragments = &document.fragments;

        let core_operations: Vec<OperationDefinitionNode> = operations
            .map(|operation| {
                let operation_kwargs = PyDict::new_bound(py);

                let operation_name = operation.name.as_ref().map(|name| self.get_name_node(py, name.as_str()));

                let operation_type = self
                    .operation_type
                    .get_operation_type(operation.operation_type);

                /*
                directives: PyTuple["DirectiveNode", ...]
                variable_definitions: Tuple["VariableDefinitionNode", ...]
                selection_set: "SelectionSetNode" */

                let directives = &operation.directives;
                let variable_definitions = &operation.variables;
                let selection_set = &operation.selection_set;

                let selection_set =
                    self.convert_selection_set_to_core_selection_set(py, selection_set);

                OperationDefinitionNode {
                    operation: operation_type,
                    name: operation_name,
                    variable_definitions: Vec::new(),
                    directives: Vec::new(),
                    selection_set,
                }
            })
            .collect();

        DocumentNode {
            definitions: core_operations,
        }
    }
}
