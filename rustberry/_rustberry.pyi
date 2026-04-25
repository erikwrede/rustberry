"""Type stubs for the native ``_rustberry`` extension module."""

from typing import Any, Sequence


class SourceLocation:
    line: int
    column: int


class GraphQLError:
    """Mirror of :class:`graphql.error.graphql_error.GraphQLError`."""

    message: str
    locations: Sequence[SourceLocation]
    path: Sequence[Any]
    extensions: Any


class Document:
    """Opaque handle to a parsed apollo-compiler executable document."""


class QueryCompiler:
    """Persistent compiler bound to a single GraphQL schema."""

    def __init__(self, schema: str) -> None: ...
    def parse(self, query: str) -> Document: ...
    def validate(self, document: Document) -> list[GraphQLError]:
        """Validate ``document`` and return any diagnostics (empty when valid)."""

    def add_validate(self, contents: str) -> list[GraphQLError]:
        """Parse and validate ``contents`` in one call."""

    def gql_core_ast(self, document: Document) -> Any:
        """Convert ``document`` into a tree of real ``graphql-core`` AST nodes."""

    def gql_core_ast_mirror(self, document: Document) -> Any:
        """Convert ``document`` into a lightweight Rust-backed mirror AST."""
