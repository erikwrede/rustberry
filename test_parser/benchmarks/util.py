import os
def get_sdl_str(test_name) -> str:
    """Return the SDL string for the test schema."""
    # read file schema.graphql

    with open(os.path.join('benchmarks/gql', test_name, 'schema.graphql')) as f:
        return f.read()


def get_query_str(test_name) -> str:
    """Return the query string for the test query."""
    # read file graphql.query
    with open(os.path.join('benchmarks/gql', test_name, 'query.gql')) as f:
        return f.read()
