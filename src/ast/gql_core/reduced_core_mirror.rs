//! Frozen Python AST mirror node classes that mimic the structure of
//! `graphql-core`'s [`graphql.language.ast`][gqlcore] node hierarchy.
//!
//! The mirror nodes implement just enough of the protocol (a `kind`
//! attribute and a fake `__class__` getter) for a Python visitor / walker
//! that performs `isinstance(node, FieldNode)` or attribute access to feel
//! at home with them. By short-circuiting the construction in Rust we save
//! the per-node overhead of calling Python class constructors.
//!
//! Class references for the `__class__` getters are loaded lazily and
//! cached in module-level [`OnceLock`]s so we never re-import
//! `graphql.language.ast` once Python first asks for it.
//!
//! [gqlcore]: https://graphql-core-3.readthedocs.io/

use std::sync::OnceLock;

use pyo3::prelude::*;
use pyo3::types::PyType;

/// Convenience alias matching pyo3 0.21's old `PyObject`. In 0.28 this is
/// just `Py<PyAny>`.
type PyObject = Py<PyAny>;

/// Resolve a class object from `graphql.language.ast` and cache it.
///
/// All mirror nodes share this helper; the `slot` is a `OnceLock` declared
/// once per node type so each class is imported at most once per process.
fn cached_class<'py>(
    py: Python<'py>,
    slot: &'static OnceLock<Py<PyType>>,
    name: &str,
) -> PyResult<Py<PyType>> {
    if let Some(cls) = slot.get() {
        return Ok(cls.clone_ref(py));
    }
    let cls: Py<PyType> = py
        .import("graphql.language.ast")?
        .getattr(name)?
        .downcast_into::<PyType>()
        .map_err(PyErr::from)?
        .unbind();
    let _ = slot.set(cls.clone_ref(py));
    Ok(cls)
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct NameNode {
    #[pyo3(get)]
    pub value: String,
}

#[pymethods]
impl NameNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "NameNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "name"
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct DocumentNode {
    #[pyo3(get)]
    pub definitions: Vec<Py<OperationDefinitionNode>>,
}

#[pymethods]
impl DocumentNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "DocumentNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "document"
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct OperationDefinitionNode {
    #[pyo3(get)]
    pub operation: PyObject,
    #[pyo3(get)]
    pub name: Option<NameNode>,
    #[pyo3(get)]
    pub directives: Vec<DirectiveNode>,
    #[pyo3(get)]
    pub variable_definitions: Vec<VariableDefinitionNode>,
    #[pyo3(get)]
    pub selection_set: Py<SelectionSetNode>,
}

#[pymethods]
impl OperationDefinitionNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "OperationDefinitionNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "operation_definition"
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct SelectionSetNode {
    #[pyo3(get)]
    pub selections: Vec<PyObject>,
}

#[pymethods]
impl SelectionSetNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "SelectionSetNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "selection_set"
    }
}

#[pyclass(frozen)]
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
    pub selection_set: Option<Py<SelectionSetNode>>,
}

#[pymethods]
impl FieldNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "FieldNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "field"
    }
}

#[pyclass(frozen)]
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
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "DirectiveNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "directive"
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct ArgumentNode {
    #[pyo3(get)]
    pub name: NameNode,
    /// One of the `*ValueNode` types.
    #[pyo3(get)]
    pub value: PyObject,
}

#[pymethods]
impl ArgumentNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "ArgumentNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "argument"
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct VariableDefinitionNode {
    #[pyo3(get)]
    pub variable: VariableNode,
    /// One of `NamedTypeNode`, `ListTypeNode`, `NonNullTypeNode`.
    #[pyo3(get)]
    pub r#type: PyObject,
    /// Optional default value (one of the `*ValueNode` variants).
    #[pyo3(get)]
    pub default_value: Option<PyObject>,
    #[pyo3(get)]
    pub directives: Vec<DirectiveNode>,
}

#[pymethods]
impl VariableDefinitionNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "VariableDefinitionNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "variable_definition"
    }
}

//#region: TypeNode
#[pyclass(frozen)]
#[derive(Clone)]
pub struct NamedTypeNode {
    #[pyo3(get)]
    pub name: NameNode,
}

#[pymethods]
impl NamedTypeNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "NamedTypeNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "named_type"
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct ListTypeNode {
    /// One of the `*TypeNode` variants.
    #[pyo3(get)]
    pub r#type: PyObject,
}

#[pymethods]
impl ListTypeNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "ListTypeNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "list_type"
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct NonNullTypeNode {
    /// `NamedTypeNode` or `ListTypeNode`.
    #[pyo3(get)]
    pub r#type: PyObject,
}

#[pymethods]
impl NonNullTypeNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "NonNullTypeNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "non_null_type"
    }
}
//#endregion: TypeNode

//#region: ValueNode
#[pyclass(frozen)]
#[derive(Clone)]
pub struct VariableNode {
    #[pyo3(get)]
    pub name: NameNode,
}

#[pymethods]
impl VariableNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "VariableNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "variable"
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct IntValueNode {
    #[pyo3(get)]
    pub value: String,
}

#[pymethods]
impl IntValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "IntValueNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "int_value"
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct FloatValueNode {
    #[pyo3(get)]
    pub value: String,
}

#[pymethods]
impl FloatValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "FloatValueNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "float_value"
    }
}

#[pyclass(frozen)]
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
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "StringValueNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "string_value"
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct BooleanValueNode {
    #[pyo3(get)]
    pub value: bool,
}

#[pymethods]
impl BooleanValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "BooleanValueNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "boolean_value"
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct NullValueNode {}

#[pymethods]
impl NullValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "NullValueNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "null_value"
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct EnumValueNode {
    #[pyo3(get)]
    pub value: String,
}

#[pymethods]
impl EnumValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "EnumValueNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "enum_value"
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct ListValueNode {
    /// `Vec` of `*ValueNode` variants.
    #[pyo3(get)]
    pub values: Vec<PyObject>,
}

#[pymethods]
impl ListValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "ListValueNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "list_value"
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct ObjectFieldNode {
    #[pyo3(get)]
    pub name: NameNode,
    /// One of the `*ValueNode` variants.
    #[pyo3(get)]
    pub value: PyObject,
}

#[pymethods]
impl ObjectFieldNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "ObjectFieldNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "object_field"
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct ObjectValueNode {
    #[pyo3(get)]
    pub fields: Vec<ObjectFieldNode>,
}

#[pymethods]
impl ObjectValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "ObjectValueNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "object_value"
    }
}
//#endregion: ValueNode

//#region: Fragments
#[pyclass(frozen)]
#[derive(Clone)]
pub struct FragmentSpreadNode {
    #[pyo3(get)]
    pub name: NameNode,
    #[pyo3(get)]
    pub directives: Vec<DirectiveNode>,
}

#[pymethods]
impl FragmentSpreadNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "FragmentSpreadNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "fragment_spread"
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct InlineFragmentNode {
    #[pyo3(get)]
    pub type_condition: Option<NamedTypeNode>,
    #[pyo3(get)]
    pub directives: Vec<DirectiveNode>,
    #[pyo3(get)]
    pub selection_set: Py<SelectionSetNode>,
}

#[pymethods]
impl InlineFragmentNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "InlineFragmentNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "inline_fragment"
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct FragmentDefinitionNode {
    #[pyo3(get)]
    pub name: NameNode,
    #[pyo3(get)]
    pub type_condition: NamedTypeNode,
    #[pyo3(get)]
    pub directives: Vec<DirectiveNode>,
    #[pyo3(get)]
    pub selection_set: Py<SelectionSetNode>,
    #[pyo3(get)]
    pub variable_definitions: Vec<VariableDefinitionNode>,
}

#[pymethods]
impl FragmentDefinitionNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "FragmentDefinitionNode")
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        "fragment_definition"
    }
}
//#endregion: Fragments
