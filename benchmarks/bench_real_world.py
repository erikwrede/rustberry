"""Benchmark rustberry against graphql-core on real-world public GraphQL schemas.

Schemas live in ``benchmarks/schemas/`` (downloaded once, see the README); the
queries live in :mod:`real_world_queries`. We measure the same matrix as the
synthetic-query benchmarks -- parse, validate, mirror conversion, full pipeline
-- but on schemas that real clients actually use.

The benchmark is tolerant: if rustberry can't load a schema (e.g. apollo-compiler
strict-validates a directive we haven't declared), we report the failure and
continue with whichever schemas do load.
"""

from __future__ import annotations

import sys
import traceback
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from _bench_utils import Stats, format_ns, format_table, measure  # noqa: E402
from real_world_queries import QUERIES_BY_SCHEMA  # noqa: E402

try:
    import rustberry
except ImportError:
    print("rustberry is not importable -- run `maturin develop --release` first")
    sys.exit(1)

try:
    from graphql import build_schema, parse as core_parse, validate as core_validate
    from graphql.language import (
        ArgumentNode,
        DirectiveNode,
        DocumentNode,
        FieldNode,
        FragmentSpreadNode,
        InlineFragmentNode,
        OperationDefinitionNode,
        SelectionSetNode,
    )
except ImportError:
    print("graphql-core is required: pip install graphql-core")
    sys.exit(1)


SCHEMA_DIR = Path(__file__).parent / "schemas"


# ---------------------------------------------------------------------- helpers


def walk_document(node: object) -> int:
    """Walk a graphql-core / rustberry-mirror DocumentNode reading attributes.

    Returns the number of nodes visited so the timer can normalise per-node.
    The walk reads exactly the attributes that an executor / formatter would.
    """
    count = 1
    if isinstance(node, DocumentNode):
        for defn in node.definitions:
            count += walk_document(defn)
        return count
    if isinstance(node, OperationDefinitionNode):
        _ = node.operation
        if node.name is not None:
            _ = node.name.value
        for vd in node.variable_definitions or ():
            count += walk_document(vd)
        for d in node.directives or ():
            count += walk_document(d)
        count += walk_document(node.selection_set)
        return count
    if isinstance(node, SelectionSetNode):
        for sel in node.selections:
            count += walk_document(sel)
        return count
    if isinstance(node, FieldNode):
        _ = node.name.value
        if node.alias is not None:
            _ = node.alias.value
        for arg in node.arguments or ():
            count += walk_document(arg)
        for d in node.directives or ():
            count += walk_document(d)
        if node.selection_set is not None:
            count += walk_document(node.selection_set)
        return count
    if isinstance(node, FragmentSpreadNode):
        _ = node.name.value
        for d in node.directives or ():
            count += walk_document(d)
        return count
    if isinstance(node, InlineFragmentNode):
        if node.type_condition is not None:
            _ = node.type_condition.name.value
        for d in node.directives or ():
            count += walk_document(d)
        count += walk_document(node.selection_set)
        return count
    if isinstance(node, ArgumentNode):
        _ = node.name.value
        # value walked shallowly; deep value walks dominate the workload
        # for fragment-heavy queries which is fine for our purposes.
        return count
    if isinstance(node, DirectiveNode):
        _ = node.name.value
        for arg in node.arguments or ():
            count += walk_document(arg)
        return count
    # Unknown node kind -- skip silently rather than crash on schemas with
    # exotic constructs (executable definitions are exhaustive in practice).
    return count


def try_load_rustberry(sdl: str) -> rustberry.QueryCompiler | None:
    try:
        return rustberry.QueryCompiler(sdl)
    except Exception as exc:  # pragma: no cover - operational path
        print(f"  rustberry FAILED to load schema: {exc}")
        return None


def try_load_core(sdl: str):
    try:
        return build_schema(sdl, assume_valid=True)
    except Exception as exc:  # pragma: no cover - operational path
        print(f"  graphql-core FAILED to build schema: {exc}")
        return None


# ----------------------------------------------------------------------- main


def benchmark_schema(schema_file: str, queries: dict[str, str]) -> None:
    sdl_path = SCHEMA_DIR / schema_file
    if not sdl_path.is_file():
        print(f"\n[skip] {schema_file}: not present at {sdl_path}")
        return

    sdl = sdl_path.read_text()
    print(f"\n=== {schema_file} ({sdl_path.stat().st_size / 1024:.1f} KB, "
          f"{sdl.count(chr(10))} lines) ===")

    compiler = try_load_rustberry(sdl)
    core_schema = try_load_core(sdl)
    if compiler is None and core_schema is None:
        print("  both backends failed to load this schema; skipping")
        return

    rows: list[tuple[str, dict[str, Stats]]] = []
    columns: list[str] = []

    for qname, qstr in queries.items():
        if qname not in columns:
            columns.append(qname)

    iters_for = {"simple": 2000, "medium": 500, "complex": 200}
    warmup_for = {"simple": 200, "medium": 100, "complex": 50}

    # ---- core baseline ----
    if core_schema is not None:
        parse_row: dict[str, Stats] = {}
        validate_row: dict[str, Stats] = {}
        full_row: dict[str, Stats] = {}
        for qname, qstr in queries.items():
            iters = iters_for.get(qname, 200)
            warmup = warmup_for.get(qname, 50)
            try:
                doc = core_parse(qstr)
            except Exception as exc:
                print(f"  graphql-core parse FAILED on {qname}: {exc}")
                continue
            parse_row[qname] = measure(lambda q=qstr: core_parse(q),
                                        warmup=warmup, iters=iters)
            validate_row[qname] = measure(
                lambda d=doc: core_validate(core_schema, d),
                warmup=warmup, iters=iters,
            )
            def full(q=qstr):
                d = core_parse(q)
                _ = core_validate(core_schema, d)
                walk_document(d)
            full_row[qname] = measure(full, warmup=warmup, iters=iters)
        rows.append(("graphql-core parse", parse_row))
        rows.append(("graphql-core validate", validate_row))
        rows.append(("graphql-core parse+validate+walk", full_row))

    # ---- rustberry ----
    if compiler is not None:
        rb_parse_row: dict[str, Stats] = {}
        rb_validate_row: dict[str, Stats] = {}
        rb_mirror_row: dict[str, Stats] = {}
        rb_full_row: dict[str, Stats] = {}
        for qname, qstr in queries.items():
            iters = iters_for.get(qname, 200)
            warmup = warmup_for.get(qname, 50)
            try:
                doc = compiler.parse(qstr)
            except Exception as exc:
                print(f"  rustberry parse FAILED on {qname}: {exc}")
                continue
            rb_parse_row[qname] = measure(lambda q=qstr: compiler.parse(q),
                                           warmup=warmup, iters=iters)
            rb_validate_row[qname] = measure(
                lambda d=doc: compiler.validate(d),
                warmup=warmup, iters=iters,
            )
            rb_mirror_row[qname] = measure(
                lambda d=doc: compiler.gql_core_ast_mirror(d),
                warmup=warmup, iters=iters,
            )
            def rb_full(q=qstr):
                d = compiler.parse(q)
                errs = compiler.validate(d)
                if errs:
                    raise RuntimeError(f"validation errors: {errs}")
                ast = compiler.gql_core_ast_mirror(d)
                walk_document(ast)
            try:
                rb_full_row[qname] = measure(rb_full, warmup=warmup, iters=iters)
            except Exception as exc:
                print(f"  rustberry full pipeline FAILED on {qname}: {exc}")
        rows.append(("rustberry parse", rb_parse_row))
        rows.append(("rustberry validate", rb_validate_row))
        rows.append(("rustberry mirror conversion", rb_mirror_row))
        rows.append(("rustberry parse+validate+mirror+walk", rb_full_row))

    # render
    print(format_table(rows, columns, label_width=40))

    # speedup summary
    if compiler is not None and core_schema is not None:
        print("\n  speedup (graphql-core / rustberry) on parse+validate+full pipeline:")
        for qname in columns:
            core_full = next(
                (r[1].get(qname) for r in rows
                 if r[0] == "graphql-core parse+validate+walk"),
                None,
            )
            rb_full = next(
                (r[1].get(qname) for r in rows
                 if r[0] == "rustberry parse+validate+mirror+walk"),
                None,
            )
            if core_full and rb_full:
                ratio = core_full.median_ns / rb_full.median_ns
                print(f"    {qname:10s}  core={format_ns(core_full.median_ns).strip()}"
                      f"  rb={format_ns(rb_full.median_ns).strip()}  ->  {ratio:5.1f}x")


def main() -> int:
    print("rustberry real-world benchmark")
    print(f"  rustberry: {getattr(rustberry, '__file__', '?')}")
    print(f"  schema dir: {SCHEMA_DIR}")
    for schema_file, queries in QUERIES_BY_SCHEMA.items():
        try:
            benchmark_schema(schema_file, queries)
        except Exception:
            print(f"\n[error] {schema_file} benchmark crashed:")
            traceback.print_exc()
    return 0


if __name__ == "__main__":
    sys.exit(main())
