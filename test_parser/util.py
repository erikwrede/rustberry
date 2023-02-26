import os


def get_sdl_str() -> str:
    """Return the SDL string for the test schema."""
    # read file graphql.schema

    with open('graphql.schema') as f:
        return f.read()
    return """
type Query {
  cat: Pet
  name: String
}

union CatOrDog = Cat | Dog

interface Pet {
  name: String
  nickname: String
}

type Dog implements Pet {
  name: String
  nickname: String
  barkVolume: Int
}

type Cat implements Pet {
  name: String
  nickname: String
  meowVolume: Int
}
    """

    with open('graphql.schema') as f:
        return f.read()

def get_query_str() -> str:
    """Return the query string for the test query."""
    # read file graphql.query
    with open('query.gql') as f:
        return f.read()
    return"""
query DEF{
  name
  cat {
    name
  }
}
"""
    with open('query.gql') as f:
        return f.read()

