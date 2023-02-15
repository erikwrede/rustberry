import timeit

from graphql import validate_schema, validate, build_ast_schema, DocumentNode
from graphql.language.parser import parse

from util import get_sdl_str, get_query_str

schema = get_sdl_str()
operation = get_query_str()

parsed_schema = parse(schema)
ast_schema = build_ast_schema(parsed_schema)

validate_schema(ast_schema)

def validate_timing():
    query = parse(operation)
    validation_errors = validate(ast_schema, query)
    print("Validation errors:", validation_errors)


num = 10
time = timeit.timeit(validate_timing, number=num)
print(f"Parsing & validation on graphql-core took an average of {time*1000 / num} milliseconds ({num} iterations)")
