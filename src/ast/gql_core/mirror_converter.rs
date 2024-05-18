use std::ops::Deref;

use apollo_compiler::{ExecutableDocument, Node};
use apollo_compiler::executable::{Argument, Field, OperationType, Selection, SelectionSet};
use apollo_compiler::schema::{Directive, Type, Value};
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

    fn convert_argument_to_core_argument(&self, py: Python, argument: &Node<Argument>) -> ArgumentNode {
        let name = self.get_name_node(py, argument.name.as_str());
        let value = self.convert_value_to_core_value(py, &argument.value);
        ArgumentNode {
            name,
            value,
        }
    }

    fn convert_field_to_core_field(&self, py: Python, field: &Node<Field>) -> FieldNode {
        let selection_set = field.selection_set.selections.first()
            .map(|_| self.convert_selection_set_to_core_selection_set(py, &field.selection_set));

        let alias = field.alias.as_ref().map(|field_alias| self.get_name_node(py, field_alias.as_str()));

        let name = self.get_name_node(py, field.name.as_str());

        let arguments = field.arguments.iter().map(|argument| {
            self.convert_argument_to_core_argument(py, argument)
        }).collect();

        let directives = field.directives.iter().map(|directive| {
            self.convert_directive_to_core_directive(py, directive)
        }).collect();

        FieldNode {
            alias,
            name,
            arguments,
            directives,
            selection_set,
        }
    }

    fn convert_selection_set_to_core_selection_set(
        &self,
        py: Python,
        selection_set: &SelectionSet,
    ) -> SelectionSetNode {
        //println!("Converting selection set...");
        let selection_set_kwargs = PyDict::new(py);
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
            selections,
        }
    }
    fn convert_type_to_core_type(&self, py: Python, ty: &Type) -> PyObject {
        match ty {
            Type::Named(named_type) => {
                let name = self.get_name_node(py, named_type.as_str());
                let core_named_type = NamedTypeNode {
                    name,
                };
                core_named_type.into_py(py)
            }
            Type::NonNullNamed(named_type) => {
                let core_named_type = NonNullTypeNode {
                    r#type: NamedTypeNode {
                        name: self.get_name_node(py, named_type.as_str()),
                    }.into_py(py),
                };
                core_named_type.into_py(py)
            }
            Type::List(list_type) => {
                let core_list_type = ListTypeNode {
                    r#type: self.convert_type_to_core_type(py, list_type),
                };
                core_list_type.into_py(py)
            }
            Type::NonNullList(list_type) => {
                let core_list_type = NonNullTypeNode {
                    r#type: ListTypeNode {
                        r#type: self.convert_type_to_core_type(py, list_type),
                    }.into_py(py),
                };
                core_list_type.into_py(py)
            }
        }
    }

    fn convert_value_to_core_value(&self, py: Python, value: &Node<Value>) -> PyObject {
        match value.deref() {
            Value::Null => {
                let core_value = NullValueNode {};
                core_value.into_py(py)
            }
            Value::Enum(name) => {
                let core_value = EnumValueNode {
                    value: name.to_string(),
                };
                core_value.into_py(py)
            }
            Value::Variable(name) => {
                let core_value = VariableNode {
                    name: self.get_name_node(py, name.as_str()),
                };
                core_value.into_py(py)
            }
            Value::String(string) => {
                let core_value = StringValueNode {
                    value: string.to_string(),
                    block: None, // FIXME do we have an equivalent in apollo rs
                };
                core_value.into_py(py)
            }
            Value::Float(float) => {
                let core_value = FloatValueNode {
                    value: float.to_string(),
                };
                core_value.into_py(py)
            }
            Value::Int(int) => {
                let core_value = IntValueNode {
                    value: int.to_string(),
                };
                core_value.into_py(py)
            }
            Value::Boolean(boolean) => {
                let core_value = BooleanValueNode {
                    value: *boolean,
                };
                core_value.into_py(py)
            }
            Value::List(values) => {
                let core_values: Vec<PyObject> = values.iter().map(|value| {
                    self.convert_value_to_core_value(py, value)
                }).collect();
                let core_value = ListValueNode {
                    values: core_values,
                };
                core_value.into_py(py)
            }
            Value::Object(fields) => {
                let core_fields: Vec<ObjectFieldNode> = fields.iter().map(|(name, value)| {
                    ObjectFieldNode {
                        name: self.get_name_node(py, name.as_str()),
                        value: self.convert_value_to_core_value(py, value),
                    }
                }).collect();
                let core_value = ObjectValueNode {
                    fields: core_fields,
                };
                core_value.into_py(py)
            }
        }
    }

    fn convert_directive_to_core_directive(&self, py: Python, directive: &Node<Directive>) -> DirectiveNode {
        let name = self.get_name_node(py, directive.name.as_str());
        let arguments = directive.arguments.iter().map(|argument| {
            self.convert_argument_to_core_argument(py, argument)
        }).collect();
        DirectiveNode {
            name,
            arguments,
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

                let directives = operation.directives.iter().map(|directive| {
                    self.convert_directive_to_core_directive(py, directive)
                }).collect();


                let variable_definitions: Vec<VariableDefinitionNode> = operation.variables.iter().map(|variable| {
                    let name = self.get_name_node(py, variable.name.as_str());
                    let variable_type = self.convert_type_to_core_type(py, variable.ty.deref());

                    let default_value = variable.default_value.as_ref().map(|value| {
                        self.convert_value_to_core_value(py, value)
                    });

                    VariableDefinitionNode {
                        variable: VariableNode {
                            name,
                        },
                        default_value,
                        directives,
                        r#type: variable_type,
                    }
                }).collect();

                let selection_set = &operation.selection_set;

                let selection_set =
                    self.convert_selection_set_to_core_selection_set(py, selection_set);

                let directives = operation.directives.iter().map(|directive| {
                    self.convert_directive_to_core_directive(py, directive)
                }).collect();

                OperationDefinitionNode {
                    operation: operation_type,
                    name: operation_name,
                    variable_definitions,
                    directives,
                    selection_set,
                }
            })
            .collect();

        DocumentNode {
            definitions: core_operations,
        }
    }
}
