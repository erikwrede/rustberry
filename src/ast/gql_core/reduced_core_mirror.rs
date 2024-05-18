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

#[pymethods]
impl DirectiveNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("DirectiveNode")?;
        Ok(field_node.into())
    }
}


#[pyclass]
#[derive(Clone)]
pub struct ArgumentNode {
    #[pyo3(get)]
    pub name: NameNode,
}

#[pymethods]
impl ArgumentNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("ArgumentNode")?;
        Ok(field_node.into())
    }
}

#[pyclass]
#[derive(Clone)]
pub struct VariableDefinitionNode {
    #[pyo3(get)]
    variable: VariableNode,
    #[pyo3(get)]
    r#type: PyObject, // of type TypeNode - NamedTypeNode, ListTypeNode, NonNullTypeNode
    #[pyo3(get)]
    default_value: Option<PyObject>, // of type ValueNode - IntValueNode, FloatValueNode, StringValueNode, BooleanValueNode, EnumValueNode, ListValueNode, ObjectValueNode, NullValueNode
    #[pyo3(get)]
    directives: Vec<DirectiveNode>,
}

#[pymethods]
impl VariableDefinitionNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("VariableDefinitionNode")?;
        Ok(field_node.into())
    }
}

//#region TypeNode
#[pyclass]
pub struct NamedTypeNode {
    #[pyo3(get)]
    name: NameNode,
}

#[pymethods]
impl NamedTypeNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("NamedTypeNode")?;
        Ok(field_node.into())
    }
}

#[pyclass]
pub struct ListTypeNode {
    #[pyo3(get)]
    r#type: PyObject, // TypeNode - NamedTypeNode, ListTypeNode, NonNullTypeNode
}

#[pymethods]
impl ListTypeNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("ListTypeNode")?;
        Ok(field_node.into())
    }
}

#[pyclass]
pub struct NonNullTypeNode {
    #[pyo3(get)]
    r#type: PyObject, // NamedTypeNode or ListTypeNode
}

#[pymethods]
impl NonNullTypeNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("NonNullTypeNode")?;
        Ok(field_node.into())
    }
}
//#endregion: TypeNode

//#region ValueNode

#[pyclass]
#[derive(Clone)]
pub struct VariableNode {
    pub name: NameNode,
}

#[pymethods]
impl VariableNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("VariableNode")?;
        Ok(field_node.into())
    }
}

#[pyclass]
struct IntValueNode {
    value: String,
}

#[pymethods]
impl IntValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("IntValueNode")?;
        Ok(field_node.into())
    }
}


#[pyclass]
struct FloatValueNode {
    value: String,
}

#[pymethods]
impl FloatValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("FloatValueNode")?;
        Ok(field_node.into())
    }
}

#[pyclass]
struct StringValueNode {
    value: String,
    block: Option<bool>,
}

#[pymethods]
impl StringValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("StringValueNode")?;
        Ok(field_node.into())
    }
}

#[pyclass]
struct BooleanValueNode {
    value: bool,
}

#[pymethods]
impl BooleanValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("BooleanValueNode")?;
        Ok(field_node.into())
    }
}

#[pyclass]
struct NullValueNode {}

#[pymethods]
impl NullValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("NullValueNode")?;
        Ok(field_node.into())
    }
}

#[pyclass]
struct EnumValueNode {
    value: String,
}

#[pymethods]
impl EnumValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("EnumValueNode")?;
        Ok(field_node.into())
    }
}

#[pyclass]
struct ListValueNode {
    values: Vec<PyObject>, //of type ValueNode
}

#[pymethods]
impl ListValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("ListValueNode")?;
        Ok(field_node.into())
    }
}

#[pyclass]
struct ObjectValueNode {
    fields: Vec<PyObject>,
}

#[pymethods]
impl ObjectValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("ObjectValueNode")?;
        Ok(field_node.into())
    }
}



//#endregion: ValueNode