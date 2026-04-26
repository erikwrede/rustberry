"""Baseline benchmark: pure ``graphql-core`` with no rustberry involved.

This file establishes the numbers that rustberry has to beat (or at least
match-with-cheaper-CPU-time) before any of paths A/B/C/D can be justified.

It measures:

1. ``parse(query)`` -- lex + parse only.
2. ``validate(schema, document)`` -- standard validation rules over a
   parsed document.
3. AST construction cost: how long it takes to build a synthetic
   ``DocumentNode`` entirely in Python via ``FieldNode(...)`` /
   ``NameNode(...)`` calls. This is the cost rustberry would pay for
   "Path B" (instantiating real graphql-core classes from Rust).
4. AST attribute-access cost: walk a parsed document and read every
   ``.name`` / ``.selection_set`` / ``.arguments`` / ``.directives``,
   reported as ns per attribute access.
5. Full pipeline parse + validate + walk.

Everything uses :func:`time.perf_counter_ns` and reports median + p95
to keep tail behaviour visible.

Run: ``python benchmarks/bench_graphql_core_baseline.py``.
"""

from __future__ import annotations

import os
import sys
from typing import Callable

# Allow running as a script: ``python benchmarks/bench_graphql_core_baseline.py``.
if __package__ in (None, ""):
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from graphql import build_schema, parse, validate  # type: ignore[import-not-found]
from graphql.language.ast import (  # type: ignore[import-not-found]
    ArgumentNode,
    DocumentNode,
    FieldNode,
    NameNode,
    OperationDefinitionNode,
    OperationType,
    SelectionSetNode,
    StringValueNode,
)

from _bench_utils import Stats, format_table, measure
from queries import QUERIES, SCHEMA_SDL

__all__ = ("main",)


# ---------------------------------------------------------------------------
# Walkers
# ---------------------------------------------------------------------------


def _walk_node(node: object) -> int:
    """Recursively read ``.kind`` and structural attributes on every node.

    Returns a counter of attribute reads so the optimiser cannot prove the
    walk is dead. Each tuple element / list element counts separately.
    """
    count = 0
    stack: list[object] = [node]
    while stack:
        n = stack.pop()
        # Read 'kind' so we exercise the equivalent of the executor's dispatch.
        count += 1
        kind = getattr(n, "kind", None)
        if kind is None:
            continue
        # Read structural attributes for the most common node shapes.
        for attr in ("name", "alias", "selection_set", "type_condition", "value"):
            child = getattr(n, attr, None)
            if child is not None:
                count += 1
                stack.append(child)
        for attr in (
            "definitions",
            "selections",
            "arguments",
            "directives",
            "variable_definitions",
            "values",
            "fields",
        ):
            children = getattr(n, attr, None)
            if children:
                for c in children:
                    count += 1
                    stack.append(c)
    return count


def _count_nodes(doc: DocumentNode) -> int:
    """Count nodes in a document; used to normalise per-node access timings."""
    return _walk_node(doc)


# ---------------------------------------------------------------------------
# Synthetic AST construction
# ---------------------------------------------------------------------------


def _build_synthetic_document(field_count: int) -> DocumentNode:
    """Build a ``DocumentNode`` with ``field_count`` top-level fields.

    Constructed entirely from graphql-core classes; no parser involved. The
    shape is chosen to roughly match the kind of trees the rustberry
    converter would emit when targeting Path B.
    """
    selections = []
    for i in range(field_count):
        selections.append(
            FieldNode(
                name=NameNode(value=f"field{i}"),
                arguments=(
                    ArgumentNode(
                        name=NameNode(value="arg"),
                        value=StringValueNode(value=f"v{i}", block=False),
                    ),
                ),
                directives=(),
                selection_set=None,
            )
        )
    return DocumentNode(
        definitions=(
            OperationDefinitionNode(
                operation=OperationType.QUERY,
                name=NameNode(value="Synth"),
                variable_definitions=(),
                directives=(),
                selection_set=SelectionSetNode(selections=tuple(selections)),
            ),
        )
    )


# ---------------------------------------------------------------------------
# Benchmark drivers
# ---------------------------------------------------------------------------


def _bench_parse() -> dict[str, Stats]:
    return {
        name: measure(lambda q=q: parse(q), warmup=200, iters=2000)
        for name, q in QUERIES.items()
    }


def _bench_validate(schema) -> dict[str, Stats]:
    docs = {name: parse(q) for name, q in QUERIES.items()}
    return {
        name: measure(
            lambda s=schema, d=doc: validate(s, d), warmup=200, iters=2000
        )
        for name, doc in docs.items()
    }


def _bench_synthetic_construction() -> dict[str, Stats]:
    """Time AST construction at multiple sizes.

    Reports per-field cost by dividing total time by number of fields.
    """
    sizes = {"Simple": 1, "Medium": 10, "Complex": 50, "Adversarial": 100}
    out: dict[str, Stats] = {}
    for name, n in sizes.items():
        # measure per call to constructor of an n-field document
        stats = measure(
            lambda n=n: _build_synthetic_document(n), warmup=50, iters=500
        )
        out[name] = stats
    return out


def _bench_walk(schema) -> dict[str, Stats]:
    """Walk parsed AST; report per-attribute-access ns by post-dividing."""
    docs = {name: parse(q) for name, q in QUERIES.items()}
    counts = {name: _count_nodes(d) for name, d in docs.items()}
    out: dict[str, Stats] = {}
    for name, doc in docs.items():
        nodes = counts[name]
        stats = measure(lambda d=doc: _walk_node(d), warmup=100, iters=1000)
        # Normalise: report ns per visited node.
        out[name] = Stats(
            samples=stats.samples,
            median_ns=stats.median_ns / nodes,
            p95_ns=stats.p95_ns / nodes,
            min_ns=stats.min_ns / nodes,
            mean_ns=stats.mean_ns / nodes,
        )
    return out


def _bench_pipeline(schema) -> dict[str, Stats]:
    out: dict[str, Stats] = {}
    for name, query in QUERIES.items():
        def run(s=schema, q=query) -> int:
            doc = parse(q)
            errs = validate(s, doc)
            assert not errs
            return _walk_node(doc)

        out[name] = measure(run, warmup=100, iters=1000)
    return out


# ---------------------------------------------------------------------------
# Entrypoint
# ---------------------------------------------------------------------------


def main() -> None:
    schema = build_schema(SCHEMA_SDL)

    # Smoke-check every query before timing so we don't time invalid documents.
    for name, query in QUERIES.items():
        doc = parse(query)
        errors = validate(schema, doc)
        if errors:
            raise SystemExit(
                f"Query {name!r} has validation errors: "
                + "; ".join(e.message for e in errors)
            )

    columns = list(QUERIES.keys())
    rows: list[tuple[str, dict[str, Stats]]] = [
        ("parse", _bench_parse()),
        ("validate", _bench_validate(schema)),
        ("construct synthetic AST", _bench_synthetic_construction()),
        ("walk AST (per-node ns)", _bench_walk(schema)),
        ("parse+validate+walk", _bench_pipeline(schema)),
    ]

    print()
    print("graphql-core baseline (median, p95)")
    print("=" * 80)
    print(format_table(rows, columns))
    print()
    print("Notes:")
    print("  * 'construct synthetic AST' builds an N-field document via")
    print("    Python class calls (no parsing). Sizes: Simple=1, Medium=10,")
    print("    Complex=50, Adversarial=100 fields.")
    print("  * 'walk AST' is normalised to ns per visited node.")
    print("  * Other rows are total ns per call.")


if __name__ == "__main__":
    main()
