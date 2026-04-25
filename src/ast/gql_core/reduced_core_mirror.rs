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
//! ## Caching strategy for hot getters
//!
//! Profiling `graphql-core`'s executor against the mirror tree showed that
//! attribute reads on small leaf nodes (`NameNode.value`, every node's
//! `kind`, `*ValueNode.value`) dominate FFI overhead. The naive
//! implementation paid a Rust→Python conversion on every getter call, so
//! reading `field.name.value` was ~28 ns versus ~10 ns for an equivalent
//! pure-Python `__slots__` class.
//!
//! To get back to parity we:
//!
//! * Store textual fields (`NameNode.value`, `IntValueNode.value`, etc.)
//!   as [`Py<PyString>`] built once at construction with
//!   [`PyString::intern`]. Interning matches `graphql-core`'s own naming
//!   convention and lets downstream `==` checks short-circuit on identity.
//! * Return [`kind`] from a module-level [`OnceLock<Py<PyString>>`] per
//!   node type so every instance shares the same interned Python string.
//!
//! With these changes the auto-derived `#[pyo3(get)]` for a
//! `Py<PyString>` field returns a cheap [`Borrowed`] view (no allocation,
//! no UTF-8 conversion) on each access.
//!
//! [gqlcore]: https://graphql-core-3.readthedocs.io/

use std::sync::OnceLock;

use pyo3::prelude::*;
use pyo3::sync::OnceLockExt;
use pyo3::types::{PyString, PyType};

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
        .cast_into::<PyType>()
        .map_err(PyErr::from)?
        .unbind();
    let _ = slot.set(cls.clone_ref(py));
    Ok(cls)
}

/// Return a borrowed handle to a process-wide interned `PyString`.
///
/// The slot is initialised on first call (via [`PyString::intern`]) and
/// reused forever after, so the per-call cost is a single relaxed atomic
/// load plus a `Bound::clone` (refcount bump). Interning at the Python
/// layer means equality checks downstream — e.g. `node.kind == "field"`
/// — can short-circuit on pointer identity.
#[inline]
pub(crate) fn cached_interned_str<'py>(
    py: Python<'py>,
    slot: &'static OnceLock<Py<PyString>>,
    text: &'static str,
) -> Bound<'py, PyString> {
    slot.get_or_init_py_attached(py, || PyString::intern(py, text).unbind())
        .bind(py)
        .clone()
}

/// Convenience macro for the `kind` getter pattern: each node type owns
/// its own static slot and a fixed string literal.
///
/// `#[pymethods]` is a proc macro that walks the literal source tokens of
/// its impl block to find `#[getter]` etc., so attributes coming out of a
/// declarative macro are invisible to it. We therefore can't define the
/// getter inside a macro and have to inline the body in each `impl`. The
/// helper [`cached_interned_str`] keeps the per-callsite boilerplate to a
/// single line.
macro_rules! kind_str {
    ($py:expr, $literal:literal) => {{
        static SLOT: ::std::sync::OnceLock<::pyo3::Py<::pyo3::types::PyString>> =
            ::std::sync::OnceLock::new();
        $crate::ast::gql_core::reduced_core_mirror::cached_interned_str($py, &SLOT, $literal)
    }};
}

#[pyclass(frozen, skip_from_py_object)]
#[derive(Clone)]
pub struct NameNode {
    /// Cached Python string for the name's text. Built once via
    /// [`PyString::intern`] at construction so the auto-derived getter
    /// only does a refcount bump per access (no Rust→Python conversion).
    #[pyo3(get)]
    pub value: Py<PyString>,
}

#[pymethods]
impl NameNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "NameNode")
    }

    #[getter]
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "name")
    }
}

#[pyclass(frozen, skip_from_py_object)]
#[derive(Clone)]
pub struct DocumentNode {
    /// Heterogeneous list of top-level definitions. Each entry is either an
    /// [`OperationDefinitionNode`] or a [`FragmentDefinitionNode`] mirror —
    /// `graphql-core`'s executor relies on fragment definitions appearing at
    /// the document root to expand `...FragmentName` selections, so we keep
    /// the Python-visible type wide enough to carry both.
    #[pyo3(get)]
    pub definitions: Vec<PyObject>,
}

#[pymethods]
impl DocumentNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "DocumentNode")
    }

    #[getter]
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "document")
    }
}

#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "operation_definition")
    }
}

#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "selection_set")
    }
}

#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "field")
    }
}

#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "directive")
    }
}

#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "argument")
    }
}

#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "variable_definition")
    }
}

//#region: TypeNode
#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "named_type")
    }
}

#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "list_type")
    }
}

#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "non_null_type")
    }
}
//#endregion: TypeNode

//#region: ValueNode
#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "variable")
    }
}

#[pyclass(frozen, skip_from_py_object)]
#[derive(Clone)]
pub struct IntValueNode {
    /// Cached Python string. `graphql-core` represents int literals as
    /// strings to preserve precision; we mirror that here and intern the
    /// value to avoid re-encoding on every getter call.
    #[pyo3(get)]
    pub value: Py<PyString>,
}

#[pymethods]
impl IntValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "IntValueNode")
    }

    #[getter]
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "int_value")
    }
}

#[pyclass(frozen, skip_from_py_object)]
#[derive(Clone)]
pub struct FloatValueNode {
    #[pyo3(get)]
    pub value: Py<PyString>,
}

#[pymethods]
impl FloatValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "FloatValueNode")
    }

    #[getter]
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "float_value")
    }
}

#[pyclass(frozen, skip_from_py_object)]
#[derive(Clone)]
pub struct StringValueNode {
    #[pyo3(get)]
    pub value: Py<PyString>,
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "string_value")
    }
}

#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "boolean_value")
    }
}

#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "null_value")
    }
}

#[pyclass(frozen, skip_from_py_object)]
#[derive(Clone)]
pub struct EnumValueNode {
    #[pyo3(get)]
    pub value: Py<PyString>,
}

#[pymethods]
impl EnumValueNode {
    #[getter(__class__)]
    pub fn __class__(&self, py: Python<'_>) -> PyResult<Py<PyType>> {
        static CLS: OnceLock<Py<PyType>> = OnceLock::new();
        cached_class(py, &CLS, "EnumValueNode")
    }

    #[getter]
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "enum_value")
    }
}

#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "list_value")
    }
}

#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "object_field")
    }
}

#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "object_value")
    }
}
//#endregion: ValueNode

//#region: Fragments
#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "fragment_spread")
    }
}

#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "inline_fragment")
    }
}

#[pyclass(frozen, skip_from_py_object)]
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
    pub fn kind<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        kind_str!(py, "fragment_definition")
    }
}
//#endregion: Fragments
