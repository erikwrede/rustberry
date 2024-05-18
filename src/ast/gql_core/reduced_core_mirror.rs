use pyo3::{Py, PyAny};
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct NameNode {
    #[pyo3(get)]
    pub value: String,
}

#[pyclass]
#[derive(Clone)]
pub struct DocumentNode {
    #[pyo3(get)]
    pub definitions: Vec<OperationDefinitionNode>,
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
    pub variable: VariableNode,
    #[pyo3(get)]
    pub r#type: PyObject, // of type TypeNode - NamedTypeNode, ListTypeNode, NonNullTypeNode
    #[pyo3(get)]
    pub default_value: Option<PyObject>, // of type ValueNode - IntValueNode, FloatValueNode, StringValueNode, BooleanValueNode, EnumValueNode, ListValueNode, ObjectValueNode, NullValueNode
    #[pyo3(get)]
    pub directives: Vec<DirectiveNode>,
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
    pub name: NameNode,
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
    pub r#type: PyObject, // TypeNode - NamedTypeNode, ListTypeNode, NonNullTypeNode
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
    pub r#type: PyObject, // NamedTypeNode or ListTypeNode
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
pub struct IntValueNode {
    pub value: String,
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
pub struct FloatValueNode {
    pub value: String,
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
pub struct StringValueNode {
    pub value: String,
    pub block: Option<bool>,
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
pub struct BooleanValueNode {
    pub value: bool,
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
pub struct NullValueNode {}

#[pymethods]
impl NullValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("NullValueNode")?;
        Ok(field_node.into())
    }
}

#[pyclass]
pub struct EnumValueNode {
    pub value: String,
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
pub struct ListValueNode {
    pub values: Vec<PyObject>, //of type ValueNode
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
pub struct ObjectFieldNode {
    pub name: NameNode,
    pub value: PyObject, //of type ValueNode
}

#[pymethods]
impl ObjectFieldNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("ObjectFieldNode")?;
        Ok(field_node.into())
    }
}

#[pyclass]
pub struct ObjectValueNode {
    pub fields: Vec<ObjectFieldNode>,
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