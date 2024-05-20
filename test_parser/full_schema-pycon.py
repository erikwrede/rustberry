import timeit
from graphql import GraphQLArgument
from graphql import execute, GraphQLObjectType, GraphQLSchema, GraphQLField, GraphQLString, GraphQLList, validate

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
from graphql.utilities import get_introspection_query, build_client_schema
from graphql import parse, print_schema

introspection_result = execute(schema, parse(get_introspection_query(True, True, True, True, True)))
schema_ast = introspection_result.data
schema_str = print_schema(build_client_schema(schema_ast))

print(schema_str)
compiler = QueryCompiler(schema_str)


def validate_timing():
    query = parse(operation3)
    validation_errors = validate(schema, query)
    # validation_success = not validation_errors
    #query = compiler.parse(operation3)
    #validation_success = compiler.validate(query)
    #query = compiler.gql_core_ast_mirror(query)
    # if not validation_success:
    #    return ExecutionResult(data=None, errors=validation_errors)

    e = execute(
        schema, query,
    )
    return e


# num = 100
# time = timeit.timeit(validate_timing, number=num)
# print(f"Execution on graphql-core took an average of {time * 1000 / num} milliseconds ({num} iterations)")

def run_benchmarks(func, warmup_time=0.1):
    # Warm up the function by calling it repeatedly for the specified time
    # start_time = time.time()
    # while time.time() < start_time + warmup_time:
    #     func()

    num = 3000
    # Measure the time taken to call the function
    time_taken = timeit.timeit(func, number=num)

    print(f"Execution on graphql-core took an average of {time_taken * 1000 / num} milliseconds ({num} iterations)")
    # Print the results
    print(f"Time taken: {time_taken:.6f} seconds")


# Example usage: benchmarking the built-in sum function
print(validate_timing())
#run_benchmarks(validate_timing, 1)
