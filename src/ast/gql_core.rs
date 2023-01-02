
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

#[pyclass(extends=Node,subclass)]
pub struct NameNode {
    #[pyo3(get)]
    value: String,
}

#[pyclass(extends=Node,subclass)]
pub struct DocumentNode {
    #[pyo3(get)]
    definitions: Arc<Vec<Arc<DefinitionNode>>>,
}

#[pyclass(extends=Node,subclass)]
pub struct DefinitionNode {

}

#[pyclass(extends=DefinitionNode,subclass)]
pub struct ExecutableDefinitionNode {
    #[pyo3(get)]
    name: Option<Arc<NameNode>>,
    #[pyo3(get)]
    directives: Arc<Vec<Arc<ConstDirectiveNode>>>,
    #[pyo3(get)]
    variable_definitions: Arc<Vec<Arc<VariableDefinitionNode>>>,
    #[pyo3(get)]
    selection_set: Arc<SelectionSetNode>,
}
#[pyclass(extends=ExecutableDefinitionNode,subclass)]
pub struct OperationDefinitionNode {
    #[pyo3(get)]
    operation: OperationType,
}

#[pyclass(extends=Node,subclass)]
pub struct VariableDefinitionNode {
    #[pyo3(get)]
    variable: Arc<VariableNode>,
    #[pyo3(get)]
    type_: Arc<TypeNode>,
    #[pyo3(get)]
    default_value: Option<Arc<ValueNode>>,
    #[pyo3(get)]
    directives: Arc<Vec<Arc<ConstDirectiveNode>>>,
}

#[pyclass(extends=Node,subclass)]
pub struct SelectionSetNode {
    #[pyo3(get)]
    selections: Arc<Vec<Arc<SelectionNode>>>,
}

#[pyclass(extends=Node,subclass)]
pub struct SelectionNode {
    #[pyo3(get)]
    directives: Arc<Vec<Arc<DirectiveNode>>>,
}

#[pyclass(extends=SelectionNode,subclass)]
pub struct FieldNode {
    #[pyo3(get)]
    alias: Option<Arc<NameNode>>,
    #[pyo3(get)]
    name: Arc<NameNode>,
    #[pyo3(get)]
    arguments: Arc<Vec<Arc<ArgumentNode>>>,
    #[pyo3(get)]
    selection_set: Option<Arc<SelectionSetNode>>,
}

#[pyclass(extends=Node,subclass)]
pub struct ArgumentNode {
    #[pyo3(get)]
    name: Arc<NameNode>,
    #[pyo3(get)]
    value: Arc<ValueNode>,
}

#[subclasses(extends=ArgumentNode,subclass)]
pub struct ConstArgumentNode {
    #[pyo3(get)]
    value: Arc<ConstValueNode>,
}

// Fragments

#[pyclass(extends=SelectionNode,subclass)]
pub struct FragmentSpreadNode {
    #[pyo3(get)]
    name: Arc<NameNode>,
}

#[pyclass(extends=SelectionNode,subclass)]
pub struct InlineFragmentNode {
    #[pyo3(get)]
    type_condition: Arc<NamedTypeNode>,
    #[pyo3(get)]
    selection_set: Arc<SelectionSetNode>,
}

#[pyclass(extends=ExecutableDefinitionNode,subclass)]
pub struct FragmentDefinitionNode {
    #[pyo3(get)]
    name: NameNode,
    #[pyo3(get)]
    type_condition: Arc<NamedTypeNode>,
}

// Values
#[pyclass(extends=Node,subclass)]
pub struct ValueNode {

}

#[pyclass(extends=ValueNode,subclass)]
pub struct VariableNode {
    #[pyo3(get)]
    name: Arc<NameNode>,
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
    values: Arc<Vec<Arc<ValueNode>>>,
}

#[pyclass(extends=ListValueNode,subclass)]
pub struct ConstListValueNode {
    #[pyo3(get)]
    values: Arc<Vec<Arc<ConstValueNode>>>,
}

#[pyclass(extends=ValueNode,subclass)]
pub struct ObjectValueNode {
    #[pyo3(get)]
    fields: Arc<Vec<Arc<ObjectFieldNode>>>,
}

#[pyclass(extends=ObjectValueNode,subclass)]
pub struct ConstObjectValueNode {
    #[pyo3(get)]
    fields: Arc<Vec<Arc<ConstObjectFieldNode>>>,
}

#[pyclass(extends=Node,subclass)]
pub struct ObjectFieldNode {
    #[pyo3(get)]
    name: Arc<NameNode>,
    #[pyo3(get)]
    value: Arc<ValueNode>,
}

#[pyclass(extends=ObjectFieldNode,subclass)]
pub struct ConstObjectFieldNode {
    #[pyo3(get)]
    value: Arc<ConstValueNode>,
}

#[pyclass(extends=Node,subclass)]
pub struct DirectiveNode {
    #[pyo3(get)]
    name: Arc<NameNode>,
    #[pyo3(get)]
    arguments: Arc<Vec<Arc<ArgumentNode>>>,
}

#[pyclass(extends=DirectiveNode,subclass)]
pub struct ConstDirectiveNode {
    #[pyo3(get)]
    arguments: Arc<Vec<Arc<ConstArgumentNode>>>,
}