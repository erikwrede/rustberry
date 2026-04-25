//! Convert an apollo-compiler [`ExecutableDocument`] into a tree of real
//! Python `graphql.language.ast` node instances by calling the Python
//! constructors. This is the slow path: it allocates a Python object for
//! every node and goes through the public dataclass constructors. Prefer
//! [`super::mirror_converter`] for hot paths.

use apollo_compiler::executable::{Field, OperationType, Selection, SelectionSet};
use apollo_compiler::{ExecutableDocument, Node};
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyDict, PyList, PyString, PyType};

type PyObject = Py<PyAny>;

/// Cached lookup of `graphql.language.ast.OperationType.{QUERY,MUTATION,SUBSCRIPTION}`.
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

    fn get<'py>(&self, py: Python<'py>, op: OperationType) -> Bound<'py, PyAny> {
        match op {
            OperationType::Query => self.query.bind(py).clone(),
            OperationType::Mutation => self.mutation.bind(py).clone(),
            OperationType::Subscription => self.subscription.bind(py).clone(),
        }
    }
}

pub struct CoreConversionContext {
    operation_type: CoreOperationType,
    operation_definition: Py<PyType>,
    selection_set_node: Py<PyType>,
    field_node: Py<PyType>,
    document_node: Py<PyType>,
    name_node: Py<PyType>,
}

impl CoreConversionContext {
    pub fn new(py: Python<'_>) -> PyResult<Self> {
        let module = py.import("graphql.language.ast")?;
        let downcast = |attr: &str| -> PyResult<Py<PyType>> {
            Ok(module
                .getattr(attr)?
                .cast_into::<PyType>()
                .map_err(PyErr::from)?
                .unbind())
        };
        let operation_type_cls = module.getattr("OperationType")?;
        Ok(Self {
            operation_type: CoreOperationType::new(&operation_type_cls)?,
            operation_definition: downcast("OperationDefinitionNode")?,
            selection_set_node: downcast("SelectionSetNode")?,
            field_node: downcast("FieldNode")?,
            document_node: downcast("DocumentNode")?,
            name_node: downcast("NameNode")?,
        })
    }

    fn get_name_node<'py>(&self, py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyAny>> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("value", PyString::new(py, name))?;
        self.name_node.bind(py).call((), Some(&kwargs))
    }

    fn convert_field<'py>(
        &self,
        py: Python<'py>,
        field: &Node<Field>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let kwargs = PyDict::new(py);
        if !field.selection_set.selections.is_empty() {
            let ss = self.convert_selection_set(py, &field.selection_set)?;
            kwargs.set_item("selection_set", ss)?;
        }
        if let Some(alias) = &field.alias {
            kwargs.set_item("alias", PyString::new(py, alias.as_str()))?;
        }
        kwargs.set_item("name", self.get_name_node(py, field.name.as_str())?)?;
        kwargs.set_item("arguments", PyList::empty(py))?;
        kwargs.set_item("directives", PyList::empty(py))?;
        self.field_node.bind(py).call((), Some(&kwargs))
    }

    fn convert_selection_set<'py>(
        &self,
        py: Python<'py>,
        selection_set: &SelectionSet,
    ) -> PyResult<Bound<'py, PyAny>> {
        let kwargs = PyDict::new(py);
        let mut items: Vec<Bound<'py, PyAny>> = Vec::with_capacity(selection_set.selections.len());
        for selection in &selection_set.selections {
            // Fragment spreads/inline fragments are intentionally skipped
            // here; the legacy converter only handles fields. Hot-path
            // callers should use `MirrorConversionContext` which has full
            // fragment support.
            if let Selection::Field(field) = selection {
                items.push(self.convert_field(py, field)?);
            }
        }
        kwargs.set_item("selections", PyList::new(py, items)?)?;
        self.selection_set_node.bind(py).call((), Some(&kwargs))
    }

    pub fn convert_core_to_core_ast<'py>(
        &self,
        py: Python<'py>,
        document: &ExecutableDocument,
    ) -> PyResult<Bound<'py, PyAny>> {
        let mut definitions: Vec<Bound<'py, PyAny>> = Vec::with_capacity(document.operations.len());
        for operation in document.operations.iter() {
            let kwargs = PyDict::new(py);
            if let Some(name) = &operation.name {
                kwargs.set_item("name", self.get_name_node(py, name.as_str())?)?;
            }
            kwargs.set_item("operation", self.operation_type.get(py, operation.operation_type))?;
            kwargs.set_item(
                "selection_set",
                self.convert_selection_set(py, &operation.selection_set)?,
            )?;
            definitions.push(self.operation_definition.bind(py).call((), Some(&kwargs))?);
        }
        let doc_kwargs = PyDict::new(py);
        doc_kwargs.set_item("definitions", PyList::new(py, definitions)?)?;
        self.document_node.bind(py).call((), Some(&doc_kwargs))
    }
}
