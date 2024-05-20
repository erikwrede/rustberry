import pytest
from graphql import validate_schema, validate, build_ast_schema
from graphql.language.parser import parse

from rustberry import QueryCompiler

from util import get_sdl_str, get_query_str


schema = get_sdl_str("gitlab")
operation = get_query_str("gitlab")

parsed_schema = parse(schema)
ast_schema = build_ast_schema(parsed_schema)
validate_schema(ast_schema)

compiler = QueryCompiler(schema)
@pytest.mark.benchmark
def test_validation_gitlab_query_graphql_core():
    query = parse(operation)
    validation_errors = validate(ast_schema, query)

@pytest.mark.benchmark
def test_validation_gitlab_query_rustberry():
    document = compiler.parse(operation)
    validation_success = compiler.validate(document)
    query = compiler.gql_core_ast_mirror(document)
