
use std::sync::Arc;
use pyo3::prelude::*;
#[pyclass]
#[derive(Debug, PartialEq, Clone)]
pub enum OperationType {
    QUERY ,
    MUTATION,
    SUBSCRIPTION,
}

#[pyclass]
pub struct Node {

}

impl Node{}

#[pyclass(extends=Node,subclass)]
pub struct NameNode {
    #[pyo3(get)]
    value: String,
}

impl NameNode {
    pub fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

#[pyclass(extends=Node,subclass)]
pub struct DocumentNode {
    #[pyo3(get)]
    definitions: Vec<DefinitionNode>,
}

#[pyclass(extends=Node,subclass)]
pub struct DefinitionNode {

}

#[pyclass(extends=DefinitionNode,subclass)]
pub struct ExecutableDefinitionNode {
    #[pyo3(get)]
    name: Option<NameNode>,
    #[pyo3(get)]
    directives: Vec<DirectiveNode>,
    #[pyo3(get)]
    variable_definitions: Vec<VariableDefinitionNode>,
    #[pyo3(get)]
    selection_set: SelectionSetNode,
}
#[pyclass(extends=ExecutableDefinitionNode,subclass)]
pub struct OperationDefinitionNode {
    #[pyo3(get)]
    operation: OperationType,
}

#[pyclass(extends=Node,subclass)]
pub struct VariableDefinitionNode {
    #[pyo3(get)]
    variable: VariableNode,
    #[pyo3(get)]
    type_: TypeNode,
    #[pyo3(get)]
    default_value: Option<ValueNode>,
    #[pyo3(get)]
    directives: Vec<DirectiveNode>,
}

#[pyclass(extends=Node,subclass)]
pub struct SelectionSetNode {
    #[pyo3(get)]
    selections: Vec<SelectionNode>,
}

#[pyclass(extends=Node,subclass)]
pub struct SelectionNode {
    #[pyo3(get)]
    directives: Vec<DirectiveNode>,
}

#[pyclass(extends=SelectionNode,subclass)]
pub struct FieldNode {
    #[pyo3(get)]
    alias: Option<NameNode>,
    #[pyo3(get)]
    name: NameNode,
    #[pyo3(get)]
    arguments: Vec<ArgumentNode>,
    #[pyo3(get)]
    selection_set: Option<SelectionSetNode>,
}

#[pyclass(extends=Node,subclass)]
pub struct ArgumentNode {
    #[pyo3(get)]
    name: NameNode,
    #[pyo3(get)]
    value: ValueNode,
}

impl ArgumentNode {
    pub fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            value: self.value.clone(),
        }
    }
}

// Fragments

#[pyclass(extends=SelectionNode,subclass)]
pub struct FragmentSpreadNode {
    #[pyo3(get)]
    name: NameNode,
}

#[pyclass(extends=SelectionNode,subclass)]
pub struct InlineFragmentNode {
    #[pyo3(get)]
    type_condition: NamedTypeNode,
    #[pyo3(get)]
    selection_set: SelectionSetNode,
}

#[pyclass(extends=ExecutableDefinitionNode,subclass)]
pub struct FragmentDefinitionNode {
    #[pyo3(get)]
    name: NameNode,
    #[pyo3(get)]
    type_condition: NamedTypeNode,
}

// Values
#[pyclass(extends=Node,subclass)]
pub struct ValueNode {

}

#[pyclass(extends=ValueNode,subclass)]
pub struct VariableNode {
    #[pyo3(get)]
    name: NameNode,
}

#[pyclass(extends=ValueNode,subclass)]
pub struct IntValueNode {
    #[pyo3(get)]
    value: String,
}

#[pyclass(extends=ValueNode,subclass)]
pub struct FloatValueNode {
    #[pyo3(get)]
    value: String,
    #[pyo3(get)]
    block: Option<bool>,
}

#[pyclass(extends=ValueNode,subclass)]
pub struct BooleanValueNode {
    #[pyo3(get)]
    value: String,
}

#[pyclass(extends=ValueNode,subclass)]
pub struct NullValueNode {
}

#[pyclass(extends=ValueNode,subclass)]
pub struct EnumValueNode {
    #[pyo3(get)]
    value: String,
}

#[pyclass(extends=ValueNode,subclass)]
pub struct ListValueNode {
    #[pyo3(get)]
    values: Vec<ValueNode>,
}


#[pyclass(extends=ValueNode,subclass)]
pub struct ObjectValueNode {
    #[pyo3(get)]
    fields: Vec<ObjectFieldNode>,
}


#[pyclass(extends=Node,subclass)]
pub struct ObjectFieldNode {
    #[pyo3(get)]
    name: NameNode,
    #[pyo3(get)]
    value: ValueNode,
}


#[pyclass(extends=Node,subclass)]
pub struct DirectiveNode {
    #[pyo3(get)]
    name: NameNode,
    #[pyo3(get)]
    arguments: Vec<ArgumentNode>,
}