use pyo3::{Py, PyAny};
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct NameNode {
    #[pyo3(get)]
    pub value: String,
}

#[pymethods]
impl NameNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let name_node = py.import("graphql.language.ast")?.getattr("NameNode")?;
        Ok(name_node.into())
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "name"
    }
}

#[pyclass]
#[derive(Clone)]
pub struct DocumentNode {
    #[pyo3(get)]
    pub definitions: Vec<OperationDefinitionNode>,
}

#[pymethods]
impl DocumentNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let document_node = py.import_bound("graphql.language.ast")?.getattr("DocumentNode")?;
        Ok(document_node.into())
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "document"
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

#[pymethods]
impl OperationDefinitionNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import("graphql.language.ast")?.getattr("OperationDefinitionNode")?;
        Ok(field_node.into())
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "operation_definition"
    }
}

#[pyclass]
#[derive(Clone)]
pub struct SelectionSetNode {
    #[pyo3(get)]
    pub selections: Vec<FieldNode>,
}

#[pymethods]
impl SelectionSetNode {
    #[getter]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("SelectionSetNode")?;
        Ok(field_node.into())
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "selection_set"
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

    #[getter]
    pub fn kind(&self) -> &'static str {
        "field"
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

    #[getter]
    pub fn kind(&self) -> &'static str {
        "directive"
    }
}

#[pyclass]
#[derive(Clone)]
pub struct ArgumentNode {
    #[pyo3(get)]
    pub name: NameNode,
    #[pyo3(get)]
    pub value: PyObject // of type ValueNode - IntValueNode, FloatValueNode, StringValueNode, BooleanValueNode, EnumValueNode, ListValueNode, ObjectValueNode, NullValueNode
}

#[pymethods]
impl ArgumentNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("ArgumentNode")?;
        Ok(field_node.into())
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "argument"
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

    #[getter]
    pub fn kind(&self) -> &'static str {
        "variable_definition"
    }
}

//#region TypeNode
#[pyclass]
#[derive(Clone)]
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

    #[getter]
    pub fn kind(&self) -> &'static str {
        "named_type"
    }
}

#[pyclass]
#[derive(Clone)]
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

    #[getter]
    pub fn kind(&self) -> &'static str {
        "list_type"
    }
}

#[pyclass]
#[derive(Clone)]
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

    #[getter]
    pub fn kind(&self) -> &'static str {
        "non_null_type"
    }
}
//#endregion: TypeNode

//#region ValueNode
#[pyclass]
#[derive(Clone)]
pub struct VariableNode {
    #[pyo3(get)]
    pub name: NameNode,
}

#[pymethods]
impl VariableNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("VariableNode")?;
        Ok(field_node.into())
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "variable"
    }
}

#[pyclass]
#[derive(Clone)]
pub struct IntValueNode {
    #[pyo3(get)]
    pub value: String,
}

#[pymethods]
impl IntValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("IntValueNode")?;
        Ok(field_node.into())
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "int_value"
    }
}

#[pyclass]
#[derive(Clone)]
pub struct FloatValueNode {
    #[pyo3(get)]
    pub value: String,
}

#[pymethods]
impl FloatValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("FloatValueNode")?;
        Ok(field_node.into())
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "float_value"
    }
}

#[pyclass]
#[derive(Clone)]
pub struct StringValueNode {
    #[pyo3(get)]
    pub value: String,
    #[pyo3(get)]
    pub block: Option<bool>,
}

#[pymethods]
impl StringValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("StringValueNode")?;
        Ok(field_node.into())
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "string_value"
    }
}

#[pyclass]
#[derive(Clone)]
pub struct BooleanValueNode {
    #[pyo3(get)]
    pub value: bool,
}

#[pymethods]
impl BooleanValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("BooleanValueNode")?;
        Ok(field_node.into())
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "boolean_value"
    }
}

#[pyclass]
#[derive(Clone)]
pub struct NullValueNode {}

#[pymethods]
impl NullValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("NullValueNode")?;
        Ok(field_node.into())
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "null_value"
    }
}

#[pyclass]
#[derive(Clone)]
pub struct EnumValueNode {
    #[pyo3(get)]
    pub value: String,
}

#[pymethods]
impl EnumValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("EnumValueNode")?;
        Ok(field_node.into())
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "enum_value"
    }
}

#[pyclass]
#[derive(Clone)]
pub struct ListValueNode {
    #[pyo3(get)]
    pub values: Vec<PyObject>, //of type ValueNode
}

#[pymethods]
impl ListValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("ListValueNode")?;
        Ok(field_node.into())
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "list_value"
    }
}

#[pyclass]
#[derive(Clone)]
pub struct ObjectFieldNode {
    #[pyo3(get)]
    pub name: NameNode,
    #[pyo3(get)]
    pub value: PyObject, //of type ValueNode
}

#[pymethods]
impl ObjectFieldNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("ObjectFieldNode")?;
        Ok(field_node.into())
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "object_field"
    }
}

#[pyclass]
#[derive(Clone)]
pub struct ObjectValueNode {
    #[pyo3(get)]
    pub fields: Vec<ObjectFieldNode>,
}

#[pymethods]
impl ObjectValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let field_node = py.import_bound("graphql.language.ast")?.getattr("ObjectValueNode")?;
        Ok(field_node.into())
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "object_value"
    }
}
//#endregion: ValueNode