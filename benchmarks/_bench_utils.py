"""Shared timing primitives for the rustberry benchmark suite.

The helpers here intentionally use :func:`time.perf_counter_ns` and expose
the per-iteration samples to the caller so we can report median + p95 (parse
times have long tails -- mean alone is misleading).

Keeping this in a private module instead of duplicating it across
``bench_*.py`` files lets every benchmark report results in a consistent
format and makes A/B comparisons trustworthy.
"""

from __future__ import annotations

from dataclasses import dataclass
from statistics import median
from time import perf_counter_ns
from typing import Callable, Iterable, Sequence

__all__ = (
    "Stats",
    "measure",
    "format_ns",
    "format_table",
)


@dataclass(frozen=True)
class Stats:
    """Distribution summary for a benchmark sample of per-iteration ns timings."""

    samples: int
    median_ns: float
    p95_ns: float
    min_ns: float
    mean_ns: float

    @classmethod
    def from_samples(cls, samples: Sequence[float]) -> "Stats":
        if not samples:
            raise ValueError("Stats.from_samples requires a non-empty sequence")
        ordered = sorted(samples)
        n = len(ordered)
        # nearest-rank p95; for small n this clamps to the max.
        p95_idx = max(0, min(n - 1, int(round(0.95 * (n - 1)))))
        return cls(
            samples=n,
            median_ns=median(ordered),
            p95_ns=ordered[p95_idx],
            min_ns=ordered[0],
            mean_ns=sum(ordered) / n,
        )


def measure(
    fn: Callable[[], object],
    *,
    warmup: int = 100,
    iters: int = 1000,
    inner: int = 1,
) -> Stats:
    """Run ``fn`` and return a :class:`Stats` summary of per-call ns timings.

    Parameters
    ----------
    fn:
        Zero-argument callable to time. The return value is ignored but kept
        live until the next iteration so dead-code elimination cannot drop it.
    warmup:
        Iterations to run untimed before measurement (lets the JIT/JIT-like
        caches warm up, lets graphql-core lazy-imports settle).
    iters:
        Number of timed iterations.
    inner:
        Number of times to call ``fn`` per timed iteration. Use ``inner > 1``
        for ops fast enough that ``perf_counter_ns`` resolution dominates --
        the per-iteration mean is recorded.
    """
    if iters <= 0:
        raise ValueError("iters must be positive")
    if inner <= 0:
        raise ValueError("inner must be positive")

    sink = None  # noqa: F841 - kept to defeat DCE
    for _ in range(warmup):
        sink = fn()

    samples: list[float] = []
    if inner == 1:
        for _ in range(iters):
            t0 = perf_counter_ns()
            sink = fn()
            samples.append(perf_counter_ns() - t0)
    else:
        # Amortise timer overhead by batching ``inner`` calls per sample.
        for _ in range(iters):
            t0 = perf_counter_ns()
            for _ in range(inner):
                sink = fn()
            samples.append((perf_counter_ns() - t0) / inner)

    return Stats.from_samples(samples)


def format_ns(value: float) -> str:
    """Render a nanosecond timing in a human-friendly unit."""
    if value < 1_000:
        return f"{value:6.1f} ns"
    if value < 1_000_000:
        return f"{value / 1_000:6.2f} us"
    if value < 1_000_000_000:
        return f"{value / 1_000_000:6.2f} ms"
    return f"{value / 1_000_000_000:6.2f} s"


def format_table(
    rows: Iterable[tuple[str, dict[str, Stats]]],
    columns: Sequence[str],
    *,
    show_p95: bool = True,
    label_width: int = 32,
) -> str:
    """Render a table of ``rows`` (label -> {column: Stats}) as a string.

    ``columns`` controls column order. Each cell shows the median; the p95 is
    shown on a follow-up line if ``show_p95`` is true.
    """
    col_width = 16
    out: list[str] = []
    header = " " * label_width + "".join(c.rjust(col_width) for c in columns)
    out.append(header)
    out.append("-" * len(header))
    for label, by_col in rows:
        median_line = label.ljust(label_width)
        p95_line = " " * label_width
        for col in columns:
            stats = by_col.get(col)
            if stats is None:
                median_line += "-".rjust(col_width)
                p95_line += "".rjust(col_width)
                continue
            median_line += format_ns(stats.median_ns).rjust(col_width)
            if show_p95:
                p95_line += f"(p95 {format_ns(stats.p95_ns).strip()})".rjust(
                    col_width
                )
        out.append(median_line)
        if show_p95:
            out.append(p95_line)
    return "\n".join(out)
