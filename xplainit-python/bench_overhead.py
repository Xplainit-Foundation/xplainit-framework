#!/usr/bin/env python3
"""
Honest performance-overhead benchmark for the Python sys.settrace() hook
(FEAT-002).

This times a CPU-bound, call-heavy workload (recursive fibonacci) with tracing
DISABLED vs ENABLED and prints the real measured overhead ratio. It does NOT
try to hit any particular target.

Reality check: fine-grained sys.settrace() tracing installs a Python-level
callback that fires on EVERY call/return/exception event, and for each traced
frame we serialize argument values and cross the PyO3 boundary into Rust. That
is inherently expensive - orders of magnitude, not percent. The PRODUCTION plan's
"<10% overhead" target is aspirational and is NOT achievable for full
call-granularity tracing. It becomes reachable only with selective tracing,
which this tracer already supports:

  * sampling_rate < 1.0        -> trace a random fraction of calls
  * include_modules={...}      -> trace only specific user modules
  * max_depth                  -> stop recording below a shallow depth
  * trace_lines=False (default)-> never install the per-line hook

The script also prints the overhead for a sampled run to illustrate how
selective tracing reduces the cost.
"""

import sys
import os
import time

_HERE = os.path.dirname(os.path.abspath(__file__))
_ROOT = os.path.dirname(_HERE)
sys.path.insert(0, os.path.join(_ROOT, 'target', 'release'))
sys.path.insert(0, os.path.join(_HERE, 'python'))

import xplainit  # noqa: E402
from tracer import AutoTracer  # noqa: E402


def workload():
    """CPU-bound, call-heavy workload: recursive fibonacci."""
    def fib(n):
        if n < 2:
            return n
        return fib(n - 1) + fib(n - 2)
    return fib(24)


def time_it(fn, repeat=3):
    """Return the best (min) wall-clock time over `repeat` runs."""
    best = float('inf')
    for _ in range(repeat):
        start = time.perf_counter()
        fn()
        elapsed = time.perf_counter() - start
        best = min(best, elapsed)
    return best


def main():
    print("=" * 70)
    print("XPLAINIT PYTHON TRACING OVERHEAD BENCHMARK (FEAT-002)")
    print("=" * 70)
    print("Workload: recursive fibonacci(24) (CPU-bound, call-heavy)\n")

    # --- Baseline: no tracing ------------------------------------------------
    baseline = time_it(workload)
    print(f"Untraced (baseline):        {baseline * 1e3:9.3f} ms")

    # --- Full tracing (every call/return recorded) ---------------------------
    backend_full = xplainit.Xplainit(enabled=True)
    tracer_full = AutoTracer(backend=backend_full, max_depth=1000)

    def traced_full():
        tracer_full.start()
        try:
            workload()
        finally:
            tracer_full.stop()

    traced = time_it(traced_full)
    full_ratio = traced / baseline if baseline > 0 else float('inf')
    full_overhead_pct = (full_ratio - 1.0) * 100.0
    print(f"Traced (full call tracing): {traced * 1e3:9.3f} ms "
          f"-> {full_ratio:6.1f}x  (+{full_overhead_pct:,.0f}%)")

    # --- Sampled tracing (selective, ~5% of calls) --------------------------
    backend_sampled = xplainit.Xplainit(enabled=True)
    tracer_sampled = AutoTracer(backend=backend_sampled, max_depth=1000,
                                sampling_rate=0.05)

    def traced_sampled():
        tracer_sampled.start()
        try:
            workload()
        finally:
            tracer_sampled.stop()

    sampled = time_it(traced_sampled)
    sampled_ratio = sampled / baseline if baseline > 0 else float('inf')
    print(f"Traced (sampling_rate=0.05):{sampled * 1e3:9.3f} ms "
          f"-> {sampled_ratio:6.1f}x")

    print("\n" + "-" * 70)
    print("HONEST RESULT")
    print("-" * 70)
    print(f"Full-call-tracing overhead: {full_ratio:.1f}x "
          f"(+{full_overhead_pct:,.0f}%) over baseline.")
    print("The plan's <10% target is NOT met by full call-granularity tracing")
    print("and is not achievable for it: once sys.settrace() is installed, a")
    print("Python-level callback fires on EVERY call/return event, which alone")
    print("dwarfs the traced workload. Note the sampled run is only marginally")
    print("faster - sampling_rate reduces how many events are RECORDED (and")
    print("cross into Rust), but the per-event Python callback still fires, so")
    print("it does not by itself get near a low overhead budget. The effective")
    print("lever is to NOT install a global settrace hook at all and instead")
    print("trace selectively (include_modules to narrow scope, shallow")
    print("max_depth, or the @trace decorators for hand-picked functions).")
    print("=" * 70)


if __name__ == "__main__":
    main()
