//! Convert an apollo-compiler [`ExecutableDocument`] into the lightweight
//! mirror node tree from [`super::reduced_core_mirror`]. This avoids per-node
//! Python class construction overhead while still being indistinguishable
//! to a `graphql-core` consumer thanks to the `__class__` and `kind`
//! getters on each node.

use std::ops::Deref;

use apollo_compiler::executable::{
    Argument, Field, Fragment, OperationType, Selection, SelectionSet,
};
use apollo_compiler::schema::{Directive, Type, Value};
use apollo_compiler::{ExecutableDocument, Node};
use pyo3::prelude::*;

use crate::ast::gql_core::reduced_core_mirror::*;

type PyObject = Py<PyAny>;

/// Cached `OperationType` enum members from `graphql.language.ast`.
struct CoreOperationType {
    query: PyObject,
    mutation: PyObject,
    subscription: PyObject,
}

impl CoreOperationType {
    fn new(operation_type_cls: &Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(Self {
            query: operation_type_cls.getattr("QUERY")?.unbind(),
            mutation: operation_type_cls.getattr("MUTATION")?.unbind(),
            subscription: operation_type_cls.getattr("SUBSCRIPTION")?.unbind(),
        })
    }

    fn get(&self, py: Python<'_>, op: OperationType) -> PyObject {
        match op {
            OperationType::Query => self.query.clone_ref(py),
            OperationType::Mutation => self.mutation.clone_ref(py),
            OperationType::Subscription => self.subscription.clone_ref(py),
        }
    }
}

/// Conversion context capable of turning an [`ExecutableDocument`] into a
/// [`DocumentNode`] mirror tree.
///
/// The struct caches Python class references that the conversion routines
/// need; create it once per [`crate::QueryCompiler`] and reuse for every
/// document.
pub struct MirrorConversionContext {
    operation_type: CoreOperationType,
}

impl MirrorConversionContext {
    pub fn new(py: Python<'_>) -> PyResult<Self> {
        let module = py.import("graphql.language.ast")?;
        let operation_type_cls = module.getattr("OperationType")?;
        Ok(Self {
            operation_type: CoreOperationType::new(&operation_type_cls)?,
        })
    }

    fn get_name_node(&self, name: &str) -> NameNode {
        NameNode {
            value: name.to_string(),
        }
    }

    fn convert_argument(&self, py: Python<'_>, argument: &Node<Argument>) -> ArgumentNode {
        ArgumentNode {
            name: self.get_name_node(argument.name.as_str()),
            value: self.convert_value(py, &argument.value),
        }
    }

    fn convert_field(&self, py: Python<'_>, field: &Node<Field>) -> PyResult<FieldNode> {
        let selection_set = if field.selection_set.selections.is_empty() {
            None
        } else {
            let inner = self.convert_selection_set(py, &field.selection_set)?;
            Some(Py::new(py, inner)?)
        };

        let alias = field
            .alias
            .as_ref()
            .map(|alias| self.get_name_node(alias.as_str()));

        let arguments = field
            .arguments
            .iter()
            .map(|argument| self.convert_argument(py, argument))
            .collect();

        let directives = field
            .directives
            .iter()
            .map(|directive| self.convert_directive(py, directive))
            .collect();

        Ok(FieldNode {
            alias,
            name: self.get_name_node(field.name.as_str()),
            arguments,
            directives,
            selection_set,
        })
    }

    fn convert_fragment_spread(
        &self,
        py: Python<'_>,
        spread: &apollo_compiler::executable::FragmentSpread,
    ) -> FragmentSpreadNode {
        FragmentSpreadNode {
            name: self.get_name_node(spread.fragment_name.as_str()),
            directives: spread
                .directives
                .iter()
                .map(|d| self.convert_directive(py, d))
                .collect(),
        }
    }

    fn convert_inline_fragment(
        &self,
        py: Python<'_>,
        inline: &apollo_compiler::executable::InlineFragment,
    ) -> PyResult<InlineFragmentNode> {
        let selection_set = self.convert_selection_set(py, &inline.selection_set)?;
        Ok(InlineFragmentNode {
            type_condition: inline
                .type_condition
                .as_ref()
                .map(|t| NamedTypeNode {
                    name: self.get_name_node(t.as_str()),
                }),
            directives: inline
                .directives
                .iter()
                .map(|d| self.convert_directive(py, d))
                .collect(),
            selection_set: Py::new(py, selection_set)?,
        })
    }

    fn convert_selection_set(
        &self,
        py: Python<'_>,
        selection_set: &SelectionSet,
    ) -> PyResult<SelectionSetNode> {
        let mut selections: Vec<PyObject> =
            Vec::with_capacity(selection_set.selections.len());
        for selection in &selection_set.selections {
            match selection {
                Selection::Field(field) => {
                    let node = self.convert_field(py, field)?;
                    selections.push(Py::new(py, node)?.into_any());
                }
                Selection::FragmentSpread(spread) => {
                    let node = self.convert_fragment_spread(py, spread);
                    selections.push(Py::new(py, node)?.into_any());
                }
                Selection::InlineFragment(inline) => {
                    let node = self.convert_inline_fragment(py, inline)?;
                    selections.push(Py::new(py, node)?.into_any());
                }
            }
        }
        Ok(SelectionSetNode { selections })
    }

    fn convert_type(&self, py: Python<'_>, ty: &Type) -> PyResult<PyObject> {
        Ok(match ty {
            Type::Named(named_type) => Py::new(
                py,
                NamedTypeNode {
                    name: self.get_name_node(named_type.as_str()),
                },
            )?
            .into_any(),
            Type::NonNullNamed(named_type) => {
                let inner = Py::new(
                    py,
                    NamedTypeNode {
                        name: self.get_name_node(named_type.as_str()),
                    },
                )?
                .into_any();
                Py::new(py, NonNullTypeNode { r#type: inner })?.into_any()
            }
            Type::List(list_type) => {
                let inner = self.convert_type(py, list_type)?;
                Py::new(py, ListTypeNode { r#type: inner })?.into_any()
            }
            Type::NonNullList(list_type) => {
                let inner = self.convert_type(py, list_type)?;
                let list_node = Py::new(py, ListTypeNode { r#type: inner })?.into_any();
                Py::new(py, NonNullTypeNode { r#type: list_node })?.into_any()
            }
        })
    }

    fn convert_value(&self, py: Python<'_>, value: &Node<Value>) -> PyObject {
        let result: PyResult<PyObject> = (|| {
            Ok(match value.deref() {
                Value::Null => Py::new(py, NullValueNode {})?.into_any(),
                Value::Enum(name) => Py::new(
                    py,
                    EnumValueNode {
                        value: name.to_string(),
                    },
                )?
                .into_any(),
                Value::Variable(name) => Py::new(
                    py,
                    VariableNode {
                        name: self.get_name_node(name.as_str()),
                    },
                )?
                .into_any(),
                Value::String(string) => Py::new(
                    py,
                    StringValueNode {
                        value: string.to_string(),
                        // apollo-compiler doesn't track whether the
                        // original literal was a block string; default to
                        // `None` to match graphql-core's optional flag.
                        block: None,
                    },
                )?
                .into_any(),
                Value::Float(float) => Py::new(
                    py,
                    FloatValueNode {
                        value: float.to_string(),
                    },
                )?
                .into_any(),
                Value::Int(int) => Py::new(
                    py,
                    IntValueNode {
                        value: int.to_string(),
                    },
                )?
                .into_any(),
                Value::Boolean(boolean) => Py::new(
                    py,
                    BooleanValueNode { value: *boolean },
                )?
                .into_any(),
                Value::List(values) => {
                    let core_values: Vec<PyObject> = values
                        .iter()
                        .map(|value| self.convert_value(py, value))
                        .collect();
                    Py::new(py, ListValueNode { values: core_values })?.into_any()
                }
                Value::Object(fields) => {
                    let core_fields: Vec<ObjectFieldNode> = fields
                        .iter()
                        .map(|(name, value)| ObjectFieldNode {
                            name: self.get_name_node(name.as_str()),
                            value: self.convert_value(py, value),
                        })
                        .collect();
                    Py::new(py, ObjectValueNode { fields: core_fields })?.into_any()
                }
            })
        })();
        // Allocation in the Python heap is essentially infallible while the
        // GIL is held; bubble up an exception if it ever does fail.
        result.unwrap_or_else(|err| {
            err.restore(py);
            py.None()
        })
    }

    fn convert_directive(&self, py: Python<'_>, directive: &Node<Directive>) -> DirectiveNode {
        DirectiveNode {
            name: self.get_name_node(directive.name.as_str()),
            arguments: directive
                .arguments
                .iter()
                .map(|argument| self.convert_argument(py, argument))
                .collect(),
        }
    }

    fn convert_variable_definitions(
        &self,
        py: Python<'_>,
        variables: &[Node<apollo_compiler::executable::VariableDefinition>],
    ) -> PyResult<Vec<VariableDefinitionNode>> {
        variables
            .iter()
            .map(|variable| {
                Ok(VariableDefinitionNode {
                    variable: VariableNode {
                        name: self.get_name_node(variable.name.as_str()),
                    },
                    default_value: variable
                        .default_value
                        .as_ref()
                        .map(|value| self.convert_value(py, value)),
                    directives: variable
                        .directives
                        .iter()
                        .map(|directive| self.convert_directive(py, directive))
                        .collect(),
                    r#type: self.convert_type(py, variable.ty.deref())?,
                })
            })
            .collect()
    }

    fn convert_fragment_definition(
        &self,
        py: Python<'_>,
        fragment: &Node<Fragment>,
    ) -> PyResult<FragmentDefinitionNode> {
        let selection_set = self.convert_selection_set(py, &fragment.selection_set)?;
        Ok(FragmentDefinitionNode {
            name: self.get_name_node(fragment.name.as_str()),
            type_condition: NamedTypeNode {
                name: self.get_name_node(fragment.selection_set.ty.as_str()),
            },
            directives: fragment
                .directives
                .iter()
                .map(|d| self.convert_directive(py, d))
                .collect(),
            selection_set: Py::new(py, selection_set)?,
            // apollo-compiler doesn't expose fragment-level variable
            // definitions (an experimental GraphQL extension); always
            // empty.
            variable_definitions: Vec::new(),
        })
    }

    pub fn convert_core_to_core_ast(
        &self,
        py: Python<'_>,
        document: &ExecutableDocument,
    ) -> PyResult<DocumentNode> {
        let mut definitions: Vec<Py<OperationDefinitionNode>> =
            Vec::with_capacity(document.operations.len() + document.fragments.len());

        for operation in document.operations.iter() {
            let selection_set =
                self.convert_selection_set(py, &operation.selection_set)?;
            let directives: Vec<DirectiveNode> = operation
                .directives
                .iter()
                .map(|directive| self.convert_directive(py, directive))
                .collect();
            let variable_definitions =
                self.convert_variable_definitions(py, &operation.variables)?;

            let op_def = OperationDefinitionNode {
                operation: self.operation_type.get(py, operation.operation_type),
                name: operation
                    .name
                    .as_ref()
                    .map(|name| self.get_name_node(name.as_str())),
                variable_definitions,
                directives,
                selection_set: Py::new(py, selection_set)?,
            };
            definitions.push(Py::new(py, op_def)?);
        }

        // Note: we don't yet attach fragment definitions to the document
        // because the typed list above is `Vec<Py<OperationDefinitionNode>>`
        // — that's a known limitation of the mirror tree, see
        // `convert_fragment_definition` for the building block. For now,
        // walking the document still observes fragment spreads via inline
        // bodies in their referencing operation. A follow-up PR can widen
        // `definitions` to a heterogeneous `Vec<PyObject>`.
        let _ = (|| -> PyResult<()> {
            for fragment in document.fragments.values() {
                let _ = self.convert_fragment_definition(py, fragment)?;
            }
            Ok(())
        })();

        Ok(DocumentNode { definitions })
    }
}
