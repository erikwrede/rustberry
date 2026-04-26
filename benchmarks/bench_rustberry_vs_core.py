"""Rustberry vs ``graphql-core`` end-to-end benchmark.

Measures the four candidate strategies for getting an executable GraphQL AST
from Rust into graphql-core's executor. Strategy nicknames match the README:

* **Path A -- mirror**: rustberry returns ``#[pyclass]`` mirror nodes that
  duck-type as graphql-core nodes. Cheap to build, expensive on every
  attribute read.
* **Path B -- python**: rustberry calls real graphql-core constructors from
  Rust. Expensive build, native attribute access afterward.
* **Path C -- hybrid / lazy**: not yet implemented in rustberry; placeholder
  for future variant. The benchmark currently skips it but keeps the column
  so results stay comparable when added.
* **Path D -- validate-only**: rustberry validates, throws away the AST,
  graphql-core re-parses for execution. The "obvious slow" baseline.

If rustberry isn't built/installable the imports fail loudly; we degrade to
a clear ``SystemExit`` instead of crashing mid-run.

Run: ``python benchmarks/bench_rustberry_vs_core.py``.
"""

from __future__ import annotations

import os
import sys
from typing import Callable

if __package__ in (None, ""):
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from graphql import build_schema, parse, validate  # type: ignore[import-not-found]

from _bench_utils import Stats, format_table, measure
from queries import QUERIES, SCHEMA_SDL

try:
    import rustberry  # type: ignore[import-not-found]
except ImportError as exc:  # pragma: no cover - exercised only when wheel missing
    rustberry = None  # type: ignore[assignment]
    _IMPORT_ERROR = exc
else:
    _IMPORT_ERROR = None


__all__ = ("main",)


# ---------------------------------------------------------------------------
# Walkers (shared with baseline -- keep semantics identical)
# ---------------------------------------------------------------------------


def _walk_node(node: object) -> int:
    """Recursively read attributes, mirroring the baseline walker.

    The walker is intentionally identical between mirror nodes and real
    graphql-core nodes -- both expose ``kind`` / ``name`` / ``selection_set``
    / etc, so any wall-clock difference is attribute-access cost.
    """
    count = 0
    stack: list[object] = [node]
    while stack:
        n = stack.pop()
        count += 1
        kind = getattr(n, "kind", None)
        if kind is None:
            continue
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


# ---------------------------------------------------------------------------
# Benchmark drivers
# ---------------------------------------------------------------------------


def _bench_compiler_construction() -> Stats:
    return measure(
        lambda: rustberry.QueryCompiler(SCHEMA_SDL),
        warmup=5,
        iters=20,
    )


def _bench_rust_parse() -> dict[str, Stats]:
    """``compiler.parse(query)`` -- Rust parser only, no Python AST."""
    out: dict[str, Stats] = {}
    for name, query in QUERIES.items():
        compiler = rustberry.QueryCompiler(SCHEMA_SDL)
        out[name] = measure(
            lambda c=compiler, q=query: c.parse(q), warmup=200, iters=2000
        )
    return out


def _bench_rust_validate() -> dict[str, Stats]:
    out: dict[str, Stats] = {}
    for name, query in QUERIES.items():
        compiler = rustberry.QueryCompiler(SCHEMA_SDL)
        doc = compiler.parse(query)
        out[name] = measure(
            lambda c=compiler, d=doc: c.validate(d), warmup=200, iters=2000
        )
    return out


def _bench_rust_add_validate() -> dict[str, Stats]:
    out: dict[str, Stats] = {}
    for name, query in QUERIES.items():
        compiler = rustberry.QueryCompiler(SCHEMA_SDL)
        out[name] = measure(
            lambda c=compiler, q=query: c.add_validate(q),
            warmup=200,
            iters=2000,
        )
    return out


def _bench_rust_to_python_path_b() -> dict[str, Stats]:
    """Path B: build full graphql-core AST via PyDict bridge."""
    out: dict[str, Stats] = {}
    for name, query in QUERIES.items():
        compiler = rustberry.QueryCompiler(SCHEMA_SDL)
        doc = compiler.parse(query)
        out[name] = measure(
            lambda c=compiler, d=doc: c.gql_core_ast(d), warmup=50, iters=500
        )
    return out


def _bench_rust_to_python_path_a() -> dict[str, Stats]:
    """Path A: build ``#[pyclass]`` mirror objects."""
    out: dict[str, Stats] = {}
    for name, query in QUERIES.items():
        compiler = rustberry.QueryCompiler(SCHEMA_SDL)
        doc = compiler.parse(query)
        out[name] = measure(
            lambda c=compiler, d=doc: c.gql_core_ast_mirror(d),
            warmup=50,
            iters=500,
        )
    return out


def _bench_walk_mirror() -> dict[str, Stats]:
    out: dict[str, Stats] = {}
    for name, query in QUERIES.items():
        compiler = rustberry.QueryCompiler(SCHEMA_SDL)
        doc = compiler.parse(query)
        mirror = compiler.gql_core_ast_mirror(doc)
        n = _walk_node(mirror)
        stats = measure(lambda m=mirror: _walk_node(m), warmup=50, iters=500)
        out[name] = Stats(
            samples=stats.samples,
            median_ns=stats.median_ns / n,
            p95_ns=stats.p95_ns / n,
            min_ns=stats.min_ns / n,
            mean_ns=stats.mean_ns / n,
        )
    return out


def _bench_walk_python() -> dict[str, Stats]:
    out: dict[str, Stats] = {}
    for name, query in QUERIES.items():
        compiler = rustberry.QueryCompiler(SCHEMA_SDL)
        doc = compiler.parse(query)
        py_ast = compiler.gql_core_ast(doc)
        n = _walk_node(py_ast)
        stats = measure(lambda p=py_ast: _walk_node(p), warmup=50, iters=500)
        out[name] = Stats(
            samples=stats.samples,
            median_ns=stats.median_ns / n,
            p95_ns=stats.p95_ns / n,
            min_ns=stats.min_ns / n,
            mean_ns=stats.mean_ns / n,
        )
    return out


def _bench_pipeline_mirror() -> dict[str, Stats]:
    out: dict[str, Stats] = {}
    for name, query in QUERIES.items():
        compiler = rustberry.QueryCompiler(SCHEMA_SDL)

        def run(c=compiler, q=query) -> int:
            doc = c.parse(q)
            errs = c.validate(doc)
            assert not errs
            mirror = c.gql_core_ast_mirror(doc)
            return _walk_node(mirror)

        out[name] = measure(run, warmup=50, iters=500)
    return out


def _bench_pipeline_python() -> dict[str, Stats]:
    out: dict[str, Stats] = {}
    for name, query in QUERIES.items():
        compiler = rustberry.QueryCompiler(SCHEMA_SDL)

        def run(c=compiler, q=query) -> int:
            doc = c.parse(q)
            errs = c.validate(doc)
            assert not errs
            py_ast = c.gql_core_ast(doc)
            return _walk_node(py_ast)

        out[name] = measure(run, warmup=50, iters=500)
    return out


def _bench_pipeline_validate_only() -> dict[str, Stats]:
    """Path D: rustberry validates, graphql-core re-parses for execution."""
    out: dict[str, Stats] = {}
    for name, query in QUERIES.items():
        compiler = rustberry.QueryCompiler(SCHEMA_SDL)

        def run(c=compiler, q=query) -> int:
            errs = c.add_validate(q)
            assert not errs
            doc = parse(q)
            return _walk_node(doc)

        out[name] = measure(run, warmup=50, iters=500)
    return out


def _bench_pipeline_graphql_core(schema) -> dict[str, Stats]:
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
# Speedup table
# ---------------------------------------------------------------------------


def _print_speedup_table(
    baseline: dict[str, Stats],
    candidates: dict[str, dict[str, Stats]],
    columns: list[str],
) -> None:
    """Print speedup of each candidate vs the graphql-core baseline."""
    label_width = 32
    col_width = 16
    header = " " * label_width + "".join(c.rjust(col_width) for c in columns)
    print()
    print("Pipeline speedup vs graphql-core (>1.0 = faster than baseline)")
    print("=" * len(header))
    print(header)
    print("-" * len(header))
    for label, by_col in candidates.items():
        line = label.ljust(label_width)
        for col in columns:
            base = baseline.get(col)
            cand = by_col.get(col)
            if base is None or cand is None:
                line += "-".rjust(col_width)
            else:
                speedup = base.median_ns / cand.median_ns
                line += f"{speedup:5.2f}x".rjust(col_width)
        print(line)


# ---------------------------------------------------------------------------
# Entrypoint
# ---------------------------------------------------------------------------


def main() -> None:
    if rustberry is None:
        raise SystemExit(
            "rustberry is not importable. Build the wheel first:\n"
            "    cd rustberry && maturin develop --release\n"
            f"original error: {_IMPORT_ERROR!r}"
        )

    schema = build_schema(SCHEMA_SDL)

    print("Compiler construction (one-time per schema):")
    print(f"  median = ", end="")
    construct_stats = _bench_compiler_construction()
    print(
        f"{construct_stats.median_ns / 1e6:.2f} ms "
        f"(p95 {construct_stats.p95_ns / 1e6:.2f} ms, "
        f"n={construct_stats.samples})"
    )

    columns = list(QUERIES.keys())

    rows: list[tuple[str, dict[str, Stats]]] = [
        ("rust parse", _bench_rust_parse()),
        ("rust validate (parsed doc)", _bench_rust_validate()),
        ("rust add_validate", _bench_rust_add_validate()),
        ("rust -> py AST (Path B)", _bench_rust_to_python_path_b()),
        ("rust -> mirror AST (Path A)", _bench_rust_to_python_path_a()),
        ("walk mirror (per-node ns)", _bench_walk_mirror()),
        ("walk py AST (per-node ns)", _bench_walk_python()),
    ]

    pipeline_baseline = _bench_pipeline_graphql_core(schema)
    pipelines: dict[str, dict[str, Stats]] = {
        "graphql-core (baseline)": pipeline_baseline,
        "rustberry mirror (Path A)": _bench_pipeline_mirror(),
        "rustberry python (Path B)": _bench_pipeline_python(),
        "rustberry validate-only (D)": _bench_pipeline_validate_only(),
    }
    rows.extend((label, table) for label, table in pipelines.items())

    print()
    print("rustberry vs graphql-core (median, p95)")
    print("=" * 80)
    print(format_table(rows, columns))

    _print_speedup_table(pipeline_baseline, pipelines, columns)
    print()
    print("Interpretation hints:")
    print("  * 'rust parse' should be <<1 us for Simple if PyO3 overhead is low.")
    print("  * Big gap between mirror and python on the *walk* row points to")
    print("    PyO3 getter overhead -- exactly what bench_pyclass_overhead.py")
    print("    isolates.")
    print("  * Path D >1.0x speedup only if rust validation is so much faster")
    print("    than graphql-core's that paying for a second parse is still worth it.")


if __name__ == "__main__":
    main()
