"""Test queries and schema for rustberry AST representation benchmarks.

The queries are sized to exercise different parser/validator/walker workloads:

* ``SIMPLE`` -- single field, no nesting; lower bound for fixed overhead.
* ``MEDIUM`` -- realistic small query with arguments, variables, one fragment.
* ``COMPLEX`` -- 50+ fields, deep nesting, multiple fragments
  (named + inline), variable definitions, directives (``@include``,
  ``@skip``, ``@deprecated``), aliased fields, multiple operations.
* ``ADVERSARIAL`` -- a deeply nested fragment chain (10+ levels) intended
  to surface validation perf cliffs around recursive fragment expansion.

The schema (:data:`SCHEMA_SDL`) is rich enough for every query to validate
against it. All values are deterministic so multiple benchmark runs see
identical inputs.
"""

from __future__ import annotations

from typing import Final

__all__ = (
    "SCHEMA_SDL",
    "SIMPLE",
    "MEDIUM",
    "COMPLEX",
    "ADVERSARIAL",
    "QUERIES",
)

SCHEMA_SDL: Final[str] = """
schema {
  query: Query
  mutation: Mutation
}

type Query {
  hello: String
  user(id: ID!): User
  users(first: Int = 10, filter: UserFilter): [User!]!
  search(query: String!, kind: SearchKind = USER): [SearchResult!]!
  node(id: ID!): Node
}

type Mutation {
  createUser(input: CreateUserInput!): User!
  updateUser(id: ID!, input: UpdateUserInput!): User!
}

interface Node {
  id: ID!
}

union SearchResult = User | Post | Comment

enum SearchKind {
  USER
  POST
  COMMENT
}

input UserFilter {
  name: String
  active: Boolean
  tags: [String!]
}

input CreateUserInput {
  name: String!
  email: String!
  age: Int
  tags: [String!]
  preferences: PreferencesInput
}

input UpdateUserInput {
  name: String
  email: String
  age: Int
  preferences: PreferencesInput
}

input PreferencesInput {
  theme: String
  notifications: Boolean
}

type User implements Node {
  id: ID!
  name: String!
  email: String!
  age: Int
  active: Boolean
  tags: [String!]!
  posts(first: Int = 5): [Post!]!
  comments: [Comment!]!
  followers: [User!]!
  following: [User!]!
  preferences: Preferences
  legacyName: String @deprecated(reason: "use name")
}

type Preferences {
  theme: String
  notifications: Boolean
  language: String
}

type Post implements Node {
  id: ID!
  title: String!
  body: String!
  author: User!
  comments: [Comment!]!
  tags: [String!]!
  createdAt: String!
}

type Comment implements Node {
  id: ID!
  body: String!
  author: User!
  post: Post!
  replies: [Comment!]!
}
"""


SIMPLE: Final[str] = "{ hello }"


MEDIUM: Final[str] = """
query GetUser($id: ID!, $includeEmail: Boolean = true) {
  user(id: $id) {
    id
    name
    email @include(if: $includeEmail)
    posts(first: 3) {
      ...PostSummary
    }
  }
}

fragment PostSummary on Post {
  id
  title
  createdAt
}
"""


COMPLEX: Final[str] = """
query DashboardQuery(
  $userId: ID!
  $postCount: Int = 10
  $includeFollowers: Boolean = true
  $skipLegacy: Boolean = false
  $searchTerm: String!
) {
  primary: user(id: $userId) {
    id
    displayName: name
    email
    age
    active
    tags
    legacyName @skip(if: $skipLegacy)
    preferences {
      theme
      notifications
      language
    }
    posts(first: $postCount) {
      ...PostFields
      author {
        id
        name
        followers @include(if: $includeFollowers) {
          id
          name
        }
      }
      comments {
        ...CommentFields
        replies {
          ...CommentFields
          author {
            ... on User {
              id
              name
              email
            }
          }
        }
      }
    }
    followers @include(if: $includeFollowers) {
      ...UserSummary
    }
    following @include(if: $includeFollowers) {
      ...UserSummary
    }
  }

  results: search(query: $searchTerm, kind: USER) {
    __typename
    ... on User {
      id
      name
      tags
    }
    ... on Post {
      id
      title
      tags
    }
    ... on Comment {
      id
      body
    }
  }

  node(id: $userId) {
    __typename
    id
    ... on User {
      name
    }
    ... on Post {
      title
    }
  }
}

mutation CreateAndFetch($input: CreateUserInput!) {
  createUser(input: $input) {
    ...UserSummary
    posts(first: 1) {
      ...PostFields
    }
  }
}

fragment UserSummary on User {
  id
  name
  email
  active
  tags
}

fragment PostFields on Post {
  id
  title
  body
  createdAt
  tags
}

fragment CommentFields on Comment {
  id
  body
  author {
    id
    name
  }
}
"""


def _build_adversarial(depth: int = 12) -> str:
    """Build a query with ``depth`` fragments chained to each other.

    Each fragment spreads the next one, forming a long chain the validator
    must walk. Used to expose perf cliffs in fragment-cycle / dependency
    analysis.
    """
    fragments: list[str] = []
    for level in range(depth):
        next_spread = (
            f"...Level{level + 1}\n      id"
            if level < depth - 1
            else "id\n      name"
        )
        fragments.append(
            f"fragment Level{level} on User {{\n      {next_spread}\n    }}"
        )
    body = "\n\n    ".join(fragments)
    return (
        "query Adversarial($id: ID!) {\n"
        "  user(id: $id) {\n"
        "    ...Level0\n"
        "  }\n"
        "}\n\n    "
        + body
        + "\n"
    )


ADVERSARIAL: Final[str] = _build_adversarial(12)


QUERIES: Final[dict[str, str]] = {
    "Simple": SIMPLE,
    "Medium": MEDIUM,
    "Complex": COMPLEX,
    "Adversarial": ADVERSARIAL,
}
