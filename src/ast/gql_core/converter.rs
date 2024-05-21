use std::ops::Deref;

use apollo_compiler::{ExecutableDocument, Node};
use apollo_compiler::ast::{Argument, Value};
use apollo_compiler::executable::{Field, OperationType, Selection, SelectionSet};
use pyo3::{PyAny, Python};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyString};
use crate::ast::gql_core::reduced_core_mirror::ArgumentNode;

struct CoreOperationType {
    Query: Py<PyAny>,
    Mutation: Py<PyAny>,
    Subscription: Py<PyAny>,
}

impl CoreOperationType {
    fn new(PyOperationType: Bound<PyAny>) -> CoreOperationType {
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
    argument_node: Py<PyAny>,

    value_node: Py<PyAny>,
    variable_node: Py<PyAny>,
    int_value_node: Py<PyAny>,
    float_value_node: Py<PyAny>,
    string_value_node: Py<PyAny>,
    boolean_value_node: Py<PyAny>,
    null_value_node: Py<PyAny>,
    enum_value_node: Py<PyAny>,
    list_value_node: Py<PyAny>,
    object_value_node: Py<PyAny>,
    object_field_node: Py<PyAny>,

}

impl CoreConversionContext {
    pub fn new(py: Python) -> Self {
        let graphql_core_ast = PyModule::import_bound(py, "graphql.language.ast").unwrap();
        let PyDocumentNode = graphql_core_ast.getattr("DocumentNode").unwrap();
        let PyOperationType = graphql_core_ast.getattr("OperationType").unwrap();
        let PyOperationDefinitionNode = graphql_core_ast.getattr("OperationDefinitionNode").unwrap();
        let PySelectionSetNode = graphql_core_ast.getattr("SelectionSetNode").unwrap();
        let PyFieldNode = graphql_core_ast.getattr("FieldNode").unwrap();
        let PyNameNode = graphql_core_ast.getattr("NameNode").unwrap();
        let PyArgumentNode = graphql_core_ast.getattr("ArgumentNode").unwrap();

        let PyValueNode = graphql_core_ast.getattr("ValueNode").unwrap();
        let PyVariableNode = graphql_core_ast.getattr("VariableNode").unwrap();
        let PyIntValueNode = graphql_core_ast.getattr("IntValueNode").unwrap();
        let PyFloatValueNode = graphql_core_ast.getattr("FloatValueNode").unwrap();
        let PyStringValueNode = graphql_core_ast.getattr("StringValueNode").unwrap();
        let PyBooleanValueNode = graphql_core_ast.getattr("BooleanValueNode").unwrap();
        let PyNullValueNode = graphql_core_ast.getattr("NullValueNode").unwrap();
        let PyEnumValueNode = graphql_core_ast.getattr("EnumValueNode").unwrap();
        let PyTupleValueNode = graphql_core_ast.getattr("ListValueNode").unwrap();
        let PyObjectValueNode = graphql_core_ast.getattr("ObjectValueNode").unwrap();
        let PyObjectFieldNode = graphql_core_ast.getattr("ObjectFieldNode").unwrap();


        Self {
            operation_type: CoreOperationType::new(PyOperationType),
            operation_definition: PyOperationDefinitionNode.into(),
            selection_set_node: PySelectionSetNode.into(),
            field_node: PyFieldNode.into(),
            document_node: PyDocumentNode.into(),
            name_node: PyNameNode.into(),
            argument_node: PyArgumentNode.into(),

            value_node: PyValueNode.into(),
            variable_node: PyVariableNode.into(),
            int_value_node: PyIntValueNode.into(),
            float_value_node: PyFloatValueNode.into(),
            string_value_node: PyStringValueNode.into(),
            boolean_value_node: PyBooleanValueNode.into(),
            null_value_node: PyNullValueNode.into(),
            enum_value_node: PyEnumValueNode.into(),
            list_value_node: PyTupleValueNode.into(),
            object_value_node: PyObjectValueNode.into(),
            object_field_node: PyObjectFieldNode.into(),
        }
    }

    fn get_name_node(&self, py: Python, name: &str) -> PyResult<PyObject> {
        let name_node_kwargs = PyDict::new_bound(py);

        let name = PyString::new_bound(py, name);
        name_node_kwargs.set_item("value", name)?;

        self.name_node.call_bound(py, (), Some(&name_node_kwargs))
    }
    fn convert_value_to_core_value(&self, py: Python, value: &Node<Value>) -> PyResult<PyObject> {
        let value_node_kwargs = PyDict::new_bound(py);

        match value.deref() {
            Value::Null => {
                self.null_value_node.call_bound(py, (), None)
            }
            Value::Enum(name) => {
                value_node_kwargs.set_item("value", name.as_str())?;
                self.enum_value_node.call_bound(py, (), Some(&value_node_kwargs))
            }
            Value::Variable(name) => {
                value_node_kwargs.set_item("name", self.get_name_node(py, name.as_str())?)?;
                self.variable_node.call_bound(py, (), Some(&value_node_kwargs))
            }
            Value::String(string) => {
                value_node_kwargs.set_item("value", string.as_str())?;
                self.string_value_node.call_bound(py, (), Some(&value_node_kwargs))
            }
            Value::Float(float) => {
                value_node_kwargs.set_item("value", float.to_string())?;
                self.float_value_node.call_bound(py, (), Some(&value_node_kwargs))
            }
            Value::Int(int) => {
                value_node_kwargs.set_item("value", int.to_string())?;
                self.int_value_node.call_bound(py, (), Some(&value_node_kwargs))
            }
            Value::Boolean(boolean) => {
                value_node_kwargs.set_item("value", boolean)?;
                self.boolean_value_node.call_bound(py, (), Some(&value_node_kwargs))
            }
            Value::List(values) => {
                let core_values: Vec<PyObject> = values.iter().map(|value| {
                    self.convert_value_to_core_value(py, value)
                }).collect::<PyResult<_>>()?;
                value_node_kwargs.set_item("values", PyTuple::new_bound(py, core_values.iter()))?;
                self.list_value_node.call_bound(py, (), Some(&value_node_kwargs))
            }
            Value::Object(fields) => {
                let core_fields: Vec<PyObject> = fields.iter().map(|(name, value)| {
                    let object_field_node_kwargs = PyDict::new_bound(py);
                    object_field_node_kwargs.set_item("name", self.get_name_node(py, name.as_str())?)?;
                    object_field_node_kwargs.set_item("value", self.convert_value_to_core_value(py, value)?)?;

                    self.object_field_node.call_bound(py, (), Some(&object_field_node_kwargs))
                }).collect::<PyResult<_>>()?;

                value_node_kwargs.set_item("fields", PyTuple::new_bound(py, core_fields.iter()))?;
                self.object_value_node.call_bound(py, (), Some(&value_node_kwargs))
            }
        }
    }


    fn convert_argument_to_core_argument(&self, py: Python, argument: &Node<Argument>) -> PyResult<PyObject> {
        let name = self.get_name_node(py, argument.name.as_str());
        let value = self.convert_value_to_core_value(py, &argument.value);
        
        
        let argument_node_kwargs = PyDict::new_bound(py);
        argument_node_kwargs.set_item("name", name?)?;
        argument_node_kwargs.set_item("value", value?)?;
        self.argument_node.call_bound(py, (), Some(&argument_node_kwargs))
    }

    fn convert_field_to_core_field(&self, py: Python, field: &Node<Field>) -> PyResult<PyObject> {
        //println!("Converting field to core field...");
        let field_node_kwargs = PyDict::new_bound(py);
        if field.selection_set.selections.len() > 0 {
            //println!("Field has selection set");
            let selection_set = self.convert_selection_set_to_core_selection_set(py, &field.selection_set)?;
            field_node_kwargs.set_item("selection_set", selection_set)?;
            //println!("Selection set converted!");
        }

        //println!("Alias");
        if let Some(alias) = &field.alias {
            field_node_kwargs.set_item("alias", self.get_name_node(py, alias.as_str())?)?;
        }

        //println!("Name");
        let name = self.get_name_node(py, field.name.as_str())?;
        field_node_kwargs.set_item("name", name)?;

        //println!("Initing lists");

        // init an empty list of pyobjects
        let arguments : Vec<PyObject> = field.arguments.iter().map(|argument| {
            self.convert_argument_to_core_argument(py, argument)
        }).collect::<PyResult<_>>()?;

        let arguments = PyTuple::new_bound(py, arguments.iter());
        let directives = PyTuple::empty_bound(py).to_object(py);


        field_node_kwargs.set_item("arguments", arguments)?;

        field_node_kwargs.set_item("directives", directives)?;

        //println!("Calling field constructor");

        self.field_node.call_bound(py, (), Some(&field_node_kwargs))
    }

    fn convert_selection_set_to_core_selection_set(&self, py: Python, selection_set: &SelectionSet) -> PyResult<PyObject> {
        //println!("Converting selection set...");
        let selection_set_kwargs = PyDict::new_bound(py);
        // FIXME do we NEED to use PyTuple here?


        let selection_vec : Vec<PyObject> = selection_set.selections.iter().map(|selection| {
            match selection {
                Selection::Field(field) => self.convert_field_to_core_field(py, field),
                Selection::FragmentSpread(_) => {
                    return Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
                        "FragmentSpread is not yet implemented",
                    ))
                }
                Selection::InlineFragment(_) => {
                    return Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
                        "InlineFragment is not yet implemented",
                    ))
                }
            }
        }).collect::<PyResult<_>>()?;

        let selections = PyTuple::new_bound(py, selection_vec.iter());

        //println!("Done converting selections!");
        selection_set_kwargs.set_item("selections", selections)?;
        //println!("Appended selections to kwargs!");
        self.selection_set_node.call_bound(py, (), Some(&selection_set_kwargs))
    }

    pub fn convert_core_to_core_ast(self: &Self, py: Python, document: &ExecutableDocument) -> PyResult<PyObject> {
        let operations = document.all_operations();
        let fragments = &document.fragments;


        let core_operations : Vec<PyObject> = operations.map(|operation| {
            let operation_kwargs = PyDict::new_bound(py);

            if let Some(operation_name) = &operation.name {
                let operation_name = self.get_name_node(py, operation_name)?;
                operation_kwargs.set_item("name", operation_name)?;
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

            operation_kwargs.set_item("operation", operation_type.into_bound(py))?;

            //println!("Operation type set kwarg!");
            operation_kwargs.set_item("selection_set", self.convert_selection_set_to_core_selection_set(py, selection_set)?)?;
            self.operation_definition.call_bound(py, (), Some(&operation_kwargs))
        }).collect::<PyResult<Vec<PyObject>>>()?;

        let document_node_kwargs = PyDict::new_bound(py);
        let py_operations = PyTuple::new_bound(py, core_operations.iter());
        document_node_kwargs.set_item("definitions", py_operations)?;

        //println!("Creating document node!");
        self.document_node.call_bound(py, (), Some(&document_node_kwargs))
    }
}