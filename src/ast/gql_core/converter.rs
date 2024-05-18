use apollo_compiler::{ExecutableDocument, Node};
use apollo_compiler::executable::{Field, OperationType, Selection, SelectionSet};
use pyo3::{PyAny, Python};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString};

struct CoreOperationType {
    Query: Py<PyAny>,
    Mutation: Py<PyAny>,
    Subscription: Py<PyAny>,
}

impl CoreOperationType {
    fn new(PyOperationType: &PyAny) -> CoreOperationType {
        let query = PyOperationType.getattr("QUERY").unwrap();
        let mutation = PyOperationType.getattr("MUTATION").unwrap();
        let subscription = PyOperationType.getattr("SUBSCRIPTION").unwrap();

        CoreOperationType {
            Query: query.into(),
            Mutation: mutation.into(),
            Subscription: subscription.into(),
        }
    }

    fn get_operation_type(&self, operation_type: OperationType) -> Py<PyAny> {
        let operation_type = match operation_type {
            OperationType::Query => self.Query.to_owned(),
            OperationType::Mutation => self.Mutation.to_owned(),
            OperationType::Subscription => self.Subscription.to_owned(),
        };
        operation_type
    }
}

pub struct CoreConversionContext {
    //graphql_core_ast: Py<PyModule>,
    operation_type: CoreOperationType,
    operation_definition: Py<PyAny>,
    selection_set_node: Py<PyAny>,
    field_node: Py<PyAny>,
    document_node: Py<PyAny>,
    name_node: Py<PyAny>,
}

impl CoreConversionContext {
    pub fn new(py: Python) -> Self {
        let graphql_core_ast = PyModule::import(py, "graphql.language.ast").unwrap();
        let PyDocumentNode = graphql_core_ast.getattr("DocumentNode").unwrap();
        let PyOperationType = graphql_core_ast.getattr("OperationType").unwrap();
        let PyOperationDefinitionNode = graphql_core_ast.getattr("OperationDefinitionNode").unwrap();
        let PySelectionSetNode = graphql_core_ast.getattr("SelectionSetNode").unwrap();
        let PyFieldNode = graphql_core_ast.getattr("FieldNode").unwrap();
        let PyNameNode = graphql_core_ast.getattr("NameNode").unwrap();

        Self {
            operation_type: CoreOperationType::new(PyOperationType),
            operation_definition: PyOperationDefinitionNode.into(),
            selection_set_node: PySelectionSetNode.into(),
            field_node: PyFieldNode.into(),
            document_node: PyDocumentNode.into(),
            name_node: PyNameNode.into(),
        }
    }

    fn get_name_nome(&self, py: Python, name: &str) -> PyResult<PyObject> {
        let name_node_kwargs = PyDict::new(py);

        let name = PyString::new(py, name);
        name_node_kwargs.set_item("value", name)?;

        self.name_node.call(py, (), Some(name_node_kwargs))
    }

    fn convert_field_to_core_field(&self, py: Python, field: &Node<Field>) -> PyResult<PyObject> {
        //println!("Converting field to core field...");
        let field_node_kwargs = PyDict::new(py);
        if field.selection_set.selections.len() > 0 {
            //println!("Field has selection set");
            let selection_set = self.convert_selection_set_to_core_selection_set(py, &field.selection_set)?;
            field_node_kwargs.set_item("selection_set", selection_set)?;
            //println!("Selection set converted!");
        }

        //println!("Alias");
        if let Some(alias) = &field.alias {
            field_node_kwargs.set_item("alias", PyString::new(py, alias.as_str()))?;
        }

        //println!("Name");
        let name = self.get_name_nome(py, field.name.as_str())?;
        field_node_kwargs.set_item("name", name)?;

        //println!("Initing lists");

        // init an empty list of pyobjects
        let arguments = PyList::empty(py).to_object(py);
        let directives = PyList::empty(py).to_object(py);


        field_node_kwargs.set_item("arguments", arguments)?;

        field_node_kwargs.set_item("directives", directives)?;

        //println!("Calling field constructor");

        self.field_node.call(py, (), Some(field_node_kwargs))
    }

    fn convert_selection_set_to_core_selection_set(&self, py: Python, selection_set: &SelectionSet) -> PyResult<PyObject> {
        //println!("Converting selection set...");
        let selection_set_kwargs = PyDict::new(py);
        // FIXME do we NEED to use PyTuple here?
        let selections = PyList::empty(py);

        for selection in &selection_set.selections {
            let core_selection = match selection {
                Selection::Field(field) => Some(self.convert_field_to_core_field(py, field)),
                Selection::FragmentSpread(fragment_spread) => None,//self.convert_fragment_spread_to_core_fragment_spread(py, fragment_spread),
                Selection::InlineFragment(inline_fragment) => None,// self.convert_inline_fragment_to_core_inline_fragment(py, inline_fragment),
            };
            if let Some(core_selection) = core_selection {
                //println!("Appending new Selection to the set...");
                selections.append(core_selection?)?;
            }
        }
        //println!("Done converting selections!");
        selection_set_kwargs.set_item("selections", selections)?;
        //println!("Appended selections to kwargs!");
        self.selection_set_node.call(py, (), Some(selection_set_kwargs))
    }

    pub fn convert_core_to_core_ast(self: &Self, py: Python, document: &ExecutableDocument) -> PyResult<PyObject> {
        let operations = document.all_operations();
        let fragments = &document.fragments;

        let core_operations = PyList::empty(py);

        for operation in operations {
            let operation_kwargs = PyDict::new(py);

            if let Some(operation_name) = &operation.name{
                let operation_name = self.get_name_nome(py, operation_name)?;
                // FIXME is this necessary?
                //println!("Trying to set name!");
                operation_kwargs.set_item("name", operation_name)?;
                //println!("Name set!");
            }


            let operation_type = self.operation_type.get_operation_type(operation.operation_type);

            //println!("Operation type resolved!");
            /*
            directives: PyTuple["DirectiveNode", ...]
            variable_definitions: Tuple["VariableDefinitionNode", ...]
            selection_set: "SelectionSetNode" */

            let directives = &operation.directives;
            let variable_definitions = &operation.variables;
            let selection_set = &operation.selection_set;

            //println!("Selection sett, directives, variables done!");

            operation_kwargs.set_item("operation", operation_type.into_ref(py))?;

            //println!("Operation type set kwarg!");
            operation_kwargs.set_item("selection_set", self.convert_selection_set_to_core_selection_set(py, selection_set)?)?;
            //println!("Selection Set converted!");

            //println!("Creating Operation def node...");
            core_operations.append(self.operation_definition.call(py, (), Some(operation_kwargs))?)?;

            //println!("Created Operation def node!");
        }
        let document_node_kwargs = PyDict::new(py);
        document_node_kwargs.set_item("definitions", core_operations)?;

        //println!("Creating document node!");
        self.document_node.call(py, (), Some(document_node_kwargs))
    }
}