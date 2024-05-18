use pyo3::{Py, PyAny};

use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct NameNode {
    #[pyo3(get)]
    pub value: String,
}

impl NameNode {
    pub fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct DocumentNode {
    #[pyo3(get)]
    pub definitions: Vec<OperationDefinitionNode>,
}

impl DocumentNode {
    pub fn clone(&self) -> Self {
        Self {
            definitions: self.definitions.clone(),
        }
    }
}


#[pyclass]
#[derive(Clone)]
pub struct OperationDefinitionNode {
    #[pyo3(get)]
    pub operation: Py<PyAny>,
    #[pyo3(get)]
    pub name: Option<NameNode>,
    #[pyo3(get)]
    pub directives: Vec<DirectiveNode>,
    #[pyo3(get)]
    pub variable_definitions: Vec<VariableDefinitionNode>,
    #[pyo3(get)]
    pub selection_set: SelectionSetNode,
}

impl OperationDefinitionNode {
    pub fn clone(&self) -> Self {
        Self {
            operation: self.operation.clone(),
            name: self.name.clone(),
            directives: self.directives.clone(),
            variable_definitions: self.variable_definitions.clone(),
            selection_set: self.selection_set.clone(),
        }
    }
}

#[pymethods]
impl OperationDefinitionNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import("graphql.language.ast")?.getattr("OperationDefinitionNode")?;
        Ok(field_node.into())
    }
}

#[pyclass]
#[derive(Clone)]
pub struct SelectionSetNode {
    #[pyo3(get)]
    pub selections: Vec<FieldNode>,
}

impl SelectionSetNode {
    pub fn clone(&self) -> Self {
        Self {
            selections: self.selections.clone(),
        }
    }
}


#[pyclass]
#[derive(Clone)]
pub struct FieldNode {
    #[pyo3(get)]
    pub directives: Vec<DirectiveNode>,
    #[pyo3(get)]
    pub alias: Option<NameNode>,
    #[pyo3(get)]
    pub name: NameNode,
    #[pyo3(get)]
    pub arguments: Vec<ArgumentNode>,
    #[pyo3(get)]
    pub selection_set: Option<SelectionSetNode>,
}

#[pymethods]
impl FieldNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("FieldNode")?;
        Ok(field_node.into())
    }
}

#[pyclass]
#[derive(Clone)]
pub struct DirectiveNode {
    #[pyo3(get)]
    pub name: NameNode,
    #[pyo3(get)]
    pub arguments: Vec<ArgumentNode>,
}


#[pyclass]
#[derive(Clone)]
pub struct ArgumentNode {
    #[pyo3(get)]
    pub name: NameNode,
}

#[pyclass]
#[derive(Clone)]
pub struct VariableDefinitionNode {
    /*#[pyo3(get)]
    variable: VariableNode,
    #[pyo3(get)]
    type_: TypeNode,
    #[pyo3(get)]
    default_value: Option<ValueNode>,
    #[pyo3(get)]
    directives: Vec<DirectiveNode>,*/
}