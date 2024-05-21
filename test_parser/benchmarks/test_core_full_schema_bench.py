import pytest
from graphql import execute, GraphQLObjectType, GraphQLArgument, GraphQLSchema, GraphQLField, GraphQLString, \
    GraphQLList, validate, ExecutionResult
from graphql.language.printer import print_ast

from rustberry import QueryCompiler


class Query:
    def resolve_hello(root, info):
        return "Hello, World!"

    def resolve_greeting(root, info):
        return f"Hello, ABC!"

    def resolve_fruits(root, info):
        return [
            {"name": "Apple"},
            {"name": "Banana"},
            {"name": "Orange"},
        ]

    def resolve_conference(root, info, id):
        print("i was called")
        # Here, you'd typically fetch conference data from a database or an API.
        # For the sake of this example, we're returning static data.
        if id == "pycon-it-2024":
            return {
                "name": "PyCon Italy 2024",
                "description": "The largest Python conference in Italy.",
                "talks": [
                    {
                        "title": "Understanding GraphQL",
                        "speaker": {
                            "name": "Jane Doe",
                            "github": "janedoe"
                        }
                    },
                    {
                        "title": "Advanced Python Techniques",
                        "speaker": {
                            "name": "John Smith",
                            "github": "johnsmith"
                        }
                    }
                ],
                "faqs": [
                    {
                        "question": "What is the date of the conference?",
                        "answer": "June 3-5, 2024"
                    },
                    {
                        "question": "Where is the conference held?",
                        "answer": "Rome, Italy"
                    }
                ]
            }
        else:
            return None


class SpeakerType(GraphQLObjectType):
    def __init__(self):
        super().__init__(
            name='Speaker',
            fields={
                'name': GraphQLField(GraphQLString),
                'github': GraphQLField(GraphQLString)
            }
        )


class TalkType(GraphQLObjectType):
    def __init__(self):
        super().__init__(
            name='Talk',
            fields={
                'title': GraphQLField(GraphQLString),
                'speaker': GraphQLField(SpeakerType())
            }
        )


class FAQType(GraphQLObjectType):
    def __init__(self):
        super().__init__(
            name='FAQ',
            fields={
                'question': GraphQLField(GraphQLString),
                'answer': GraphQLField(GraphQLString)
            }
        )


class ConferenceType(GraphQLObjectType):
    def __init__(self):
        super().__init__(
            name='Conference',
            fields={
                'name': GraphQLField(GraphQLString),
                'description': GraphQLField(GraphQLString),
                'talks': GraphQLField(GraphQLList(TalkType())),
                'faqs': GraphQLField(GraphQLList(FAQType()))
            }
        )


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
                'hellob': GraphQLField(GraphQLString, resolve=Query.resolve_hello),
                'helloc': GraphQLField(GraphQLString, resolve=Query.resolve_hello),
                'hellod': GraphQLField(GraphQLString, resolve=Query.resolve_hello),
                'helloe': GraphQLField(GraphQLString, resolve=Query.resolve_hello),
                'hellof': GraphQLField(GraphQLString, resolve=Query.resolve_hello),
                'hellog': GraphQLField(GraphQLString, resolve=Query.resolve_hello),
                'helloh': GraphQLField(GraphQLString, resolve=Query.resolve_hello),
                'greeting': GraphQLField(GraphQLString, resolve=Query.resolve_greeting),
                'fruits': GraphQLField(GraphQLList(FruitType()), resolve=Query.resolve_fruits),
                'conference': GraphQLField(
                    ConferenceType(),
                    args={
                        'id': GraphQLArgument(GraphQLString)
                    },
                    resolve=Query.resolve_conference
                )
            }
        )


schema = GraphQLSchema(query=QueryType())

from graphql import validate_schema

validate_schema(schema)
operation2 = """
        { hello, greeting, fruits { name } }
        """
operation = """
        { hello,hellob,helloc,hellod,helloe,hellof,hellog,helloh,b:hello,c:hello,d:hello greeting, fruits { name } }
        """
operation3 = """
query ConferenceQuery {
	conference(id: "pycon-it-2024") {
	name
	description	
	talks {
		title
		speaker {
			name
			github
		}
	}
	faqs {
		question
		answer
	}
	}	
} 	

"""
operation_large = """
query ConferenceQuery {
	a: conference(id: "pycon-it-2024") {
	name
	description	
	talks {
		title
		speaker {
			name
			github
		}
	}
	faqs {
		question
		answer
	}
	}
	b: conference(id: "pycon-it-2024") {
	name
	description	
	talks {
		title
		speaker {
			name
			github
		}
	}
	faqs {
		question
		answer
	}
	}
	c: conference(id: "pycon-it-2024") {
	name
	description	
	talks {
		title
		speaker {
			name
			github
		}
	}
	faqs {
		question
		answer
	}
	}
	d: conference(id: "pycon-it-2024") {
	name
	description	
	talks {
		title
		speaker {
			name
			github
		}
	}
	faqs {
		question
		answer
	}
	}	
} 	

"""

from graphql.utilities import get_introspection_query, build_client_schema
from graphql import parse, print_schema

introspection_result = execute(schema, parse(get_introspection_query(True, True, True, True, True)))
schema_ast = introspection_result.data
schema_str = print_schema(build_client_schema(schema_ast))

print(schema_str)
compiler = QueryCompiler(schema_str)


@pytest.mark.benchmark
def test_pycon_query_execution_graphql_core():
    query = parse(operation)
    validation_errors = validate(schema, query)
    validation_success = not validation_errors

    if not validation_success:
        return ExecutionResult(data=None, errors=validation_errors)

    e = execute(
        schema, query,
    )


@pytest.mark.benchmark
def test_pycon_query_execution_rustberry():
    document = compiler.parse(operation)
    validation_success = compiler.validate(document)
    query = compiler.gql_core_ast_mirror(document)
    if not validation_success:
        return ExecutionResult(data=None, errors=validation_errors)

    e = execute(
        schema, query,
    )
@pytest.mark.benchmark
def test_pycon_query_execution_rustberry_no_mirror():
    document = compiler.parse(operation)
    validation_success = compiler.validate(document)
    query = compiler.gql_core_ast_mirror(document)
    if not validation_success:
        return ExecutionResult(data=None, errors=validation_errors)

    e = execute(
        schema, query,
    )

@pytest.mark.benchmark
def test_pure_execution_core(benchmark):
    query = parse(operation)
    benchmark(execute, schema, query)

@pytest.mark.benchmark
def test_pure_execution_rustberry(benchmark):
    document = compiler.parse(operation)
    query = compiler.gql_core_ast_mirror(document)
    benchmark(execute, schema, query)
@pytest.mark.benchmark
def test_pure_execution_rustberry_no_mirror(benchmark):
    document = compiler.parse(operation)
    query = compiler.gql_core_ast(document)
    benchmark(execute, schema, query)



@pytest.mark.benchmark
def test_pure_execution_core_large(benchmark):
    query = parse(operation_large)
    benchmark(execute, schema, query)

@pytest.mark.benchmark
def test_pure_execution_rustberry_large(benchmark):
    document = compiler.parse(operation_large)
    query = compiler.gql_core_ast_mirror(document)
    benchmark(execute, schema, query)

@pytest.mark.benchmark
def test_pure_execution_rustberry_large_no_mirrpr(benchmark):
    document = compiler.parse(operation_large)
    query = compiler.gql_core_ast(document)
    #print(print_ast(query))
    benchmark(execute, schema, query)
