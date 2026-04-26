"""Micro-benchmark: how expensive is a PyO3 ``#[getter]`` call?

The hypothesis under test: ``#[pyclass]`` getter dispatch has a measurable
per-call overhead compared to ordinary Python attribute access. If true,
returning rustberry mirror nodes that the executor will read tens of
thousands of times will be slower than returning real graphql-core nodes.

We compare the ns/access cost of:

* ``rustberry.NameNode.value`` -- PyO3 ``#[getter]`` returning a ``String``
  (read from rustberry's own mirror types).
* ``graphql.language.ast.NameNode.value`` -- pure Python class with
  ``__slots__`` (and a ``__dict__`` slot for compatibility).
* A bespoke ``__slots__``-only Python class -- to isolate the cost added
  by graphql-core's ``__dict__`` slot and ``__init_subclass__`` machinery.
* A trivial Python ``object`` with an attribute -- the cheapest case.

Limitations
-----------
We don't ship a stand-alone PyO3 crate just for this benchmark; we
piggyback on rustberry's pre-existing mirror types. If rustberry isn't
built we still run the pure-Python comparison so the file is useful in
isolation, with a printed note that the PyO3 row was skipped.
"""

from __future__ import annotations

import os
import sys

if __package__ in (None, ""):
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from graphql.language.ast import NameNode as CoreNameNode  # type: ignore[import-not-found]

from _bench_utils import Stats, format_ns, measure

try:
    import rustberry  # type: ignore[import-not-found]

    # Mirror types are not exposed at the top level; reach into the
    # extension module for the benchmark only.
    from rustberry import _rustberry as _rb_ext  # type: ignore[attr-defined]
except ImportError as exc:  # pragma: no cover
    rustberry = None  # type: ignore[assignment]
    _rb_ext = None  # type: ignore[assignment]
    _RB_ERR = exc
else:
    _RB_ERR = None


# ---------------------------------------------------------------------------
# Comparison classes
# ---------------------------------------------------------------------------


class SlotsName:
    """Plain ``__slots__``-only Python class for comparison."""

    __slots__ = ("value",)

    def __init__(self, value: str) -> None:
        self.value = value


class DictName:
    """Plain ``__dict__``-backed Python class for comparison."""

    def __init__(self, value: str) -> None:
        self.value = value


# ---------------------------------------------------------------------------
# Benchmark
# ---------------------------------------------------------------------------


def _bench_attribute_read(obj: object, *, inner: int = 1024) -> Stats:
    """Read ``obj.value`` ``inner`` times per timed iteration.

    Using ``inner=1024`` amortises ``perf_counter_ns`` resolution -- on macOS
    that's about 41 ns per call, which is the same order of magnitude as the
    fastest attribute reads we want to time. Without batching the timer noise
    would swamp the signal.
    """
    # Define the loop in a local closure so attribute lookup of ``inner`` is
    # compiled into a constant by CPython's peepholer.
    def loop(o=obj, n=inner):
        v = None
        for _ in range(n):
            v = o.value
        return v

    stats = measure(loop, warmup=200, iters=2000)
    return Stats(
        samples=stats.samples,
        median_ns=stats.median_ns / inner,
        p95_ns=stats.p95_ns / inner,
        min_ns=stats.min_ns / inner,
        mean_ns=stats.mean_ns / inner,
    )


def _make_rustberry_name() -> object | None:
    """Return a rustberry NameNode-like object, or ``None`` if unbuildable.

    The cleanest source of a rustberry mirror NameNode is to parse a tiny
    query and harvest one from the resulting tree. This avoids any private
    API of the extension module.
    """
    if rustberry is None:
        return None
    sdl = "type Query { hello: String }"
    compiler = rustberry.QueryCompiler(sdl)
    doc = compiler.parse("{ hello }")
    mirror = compiler.gql_core_ast_mirror(doc)
    # Drill in: DocumentNode -> definitions[0] -> selection_set ->
    # selections[0] -> name
    operation = mirror.definitions[0]
    field = operation.selection_set.selections[0]
    return field.name


def _print_row(label: str, stats: Stats) -> None:
    print(
        f"  {label:<38}"
        f" median={format_ns(stats.median_ns):>10}"
        f"  p95={format_ns(stats.p95_ns):>10}"
        f"  min={format_ns(stats.min_ns):>10}"
    )


def main() -> None:
    print("PyO3 #[pyclass] getter vs Python attribute access")
    print("=" * 64)

    rb_name = _make_rustberry_name()
    if rb_name is None:
        print("  [skipped] rustberry not importable; pure-Python results only")
        print(f"            ({_RB_ERR!r})")

    print()
    print("Reading obj.value, ns per access (1024 reads/iter, 2000 iters):")
    print()

    if rb_name is not None:
        _print_row("rustberry mirror NameNode", _bench_attribute_read(rb_name))

    _print_row(
        "graphql-core NameNode (slots+dict)",
        _bench_attribute_read(CoreNameNode(value="x")),
    )
    _print_row("Python __slots__ class", _bench_attribute_read(SlotsName("x")))
    _print_row("Python __dict__ class", _bench_attribute_read(DictName("x")))

    # A SimpleNamespace-style object for completeness.
    obj = type("X", (), {"value": "x"})()
    _print_row("class attribute (no instance)", _bench_attribute_read(obj))

    print()
    print("Hypothesis check:")
    print("  If 'rustberry mirror NameNode' is materially slower than")
    print("  'graphql-core NameNode', PyO3 getter overhead dominates and")
    print("  Path A's downside is real. If the two are within ~20%, the")
    print("  AST-creation savings of Path A win.")


if __name__ == "__main__":
    main()
