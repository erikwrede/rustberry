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
#compiler.validate()

def validate_timing():
    file_id = compiler.add_executable(operation)
    validated_successfully = compiler.validate_file(file_id)
    query = compiler.gql_core_ast_mirror(file_id)


    if (not validated_successfully):
        print("Validation errors!")

    e =  execute(
        schema, query,
    )



def run_benchmarks(func, warmup_time=0.1):
    # Warm up the function by calling it repeatedly for the specified time
    start_time = time.time()
    while time.time() < start_time + warmup_time:
        func()

    num = 100
    # Measure the time taken to call the function
    time_taken = timeit.timeit(func, number=num)

    print(f"Execution on graphql-core took an average of {time_taken * 1000 / num} milliseconds ({num} iterations)")
    # Print the results
    print(f"Time taken: {time_taken:.6f} seconds")

# Example usage: benchmarking the built-in sum function

run_benchmarks(validate_timing, 1)