import timeit

from graphql import execute, GraphQLObjectType, GraphQLSchema, GraphQLField, GraphQLString, GraphQLList

from rustberry import QueryCompiler


class Query:
    def resolve_hello(root, info):
        return "Hello, World!"

    def resolve_greeting(root, info):
        return f"Hello, ABC!"

    def resolve_fruits(root, info):
        return [
            {
                "name": "Apple",
            },
            {
                "name": "Banana",
            },
            {
                "name": "Orange",
            },
        ]


class FruitType(GraphQLObjectType):
    def __init__(self):
        super().__init__(
            name='Fruit',
            fields={
                'name': GraphQLField(GraphQLString)
            }
        )


class QueryType(GraphQLObjectType):
    def __init__(self):
        super().__init__(
            name='Query',
            fields={
                'hello': GraphQLField(GraphQLString, resolve=Query.resolve_hello),
                'greeting': GraphQLField(GraphQLString, resolve=Query.resolve_greeting),
                'fruits': GraphQLField(GraphQLList(FruitType()), resolve=Query.resolve_fruits)
            }
        )


schema = GraphQLSchema(query=QueryType())

from graphql import validate_schema

validate_schema(schema)
operation = """
        { hello, greeting, fruits { name } }
        """

from graphql.language.printer import print_ast
from graphql.utilities import get_introspection_query, build_client_schema
from graphql import parse, print_schema, validate, ExecutionResult

introspection_result = execute(schema, parse(get_introspection_query(True,True,True,True,True)))
schema_ast = introspection_result.data
schema_str = print_schema(build_client_schema(schema_ast))

print(schema_str)
compiler = QueryCompiler()
compiler.set_schema(schema_str)


def validate_timing():
    query = parse(operation)
    validation_errors = validate(schema, query)

    if validation_errors:
        return ExecutionResult(data=None, errors=validation_errors)



    e =  execute(
        schema, query,
    )


num = 100
time = timeit.timeit(validate_timing, number=num)
print(f"Execution on graphql-core took an average of {time * 1000 / num} milliseconds ({num} iterations)")
