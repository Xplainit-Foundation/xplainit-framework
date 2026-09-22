"""
Revert-sensitive tests for the xplainit PUBLIC convenience API.

These exercise the exact top-level entry points a naive user reaches for after
`import xplainit`, and were written to FAIL if any of the three audited fixes
is reverted:

  1. The compiled-module ``xplainit.AutoTracer`` actually installs
     ``sys.settrace()`` and captures real events (defect #1: was a no-op that
     only flipped a flag and returned the JSON string "[]").
  2. ``xplainit.auto_trace()`` enables working tracing without raising
     (defect #2: raised ``ModuleNotFoundError: No module named 'xplainit.python'``
     because the helper package was never shipped in the wheel).
  3. ``@xplainit.explain_function`` yields a callable that returns the correct
     result AND records a trace (defect #3: ``eval`` of a ``def`` raised
     ``SyntaxError`` which was swallowed to ``None``, so the decorated function
     became ``None`` and calling it raised ``TypeError``).

Run:  python xplainit-python/test_public_api.py
"""

import json
import os
import sys

# Import the installed compiled module (maturin develop installs it).
import xplainit


def _events_list(obj):
    """Normalize get_events() output (list already, or JSON string) to a list."""
    if isinstance(obj, str):
        return json.loads(obj)
    return list(obj)


def _count_enters(events, name):
    return sum(
        1
        for e in events
        if isinstance(e, dict)
        and "FunctionEnter" in e
        and e["FunctionEnter"].get("name") == name
    )


def _count_exceptions(events):
    return sum(1 for e in events if isinstance(e, dict) and "Exception" in e)


# ---------------------------------------------------------------------------
# Defect #1: native AutoTracer must capture real events (not be a silent no-op)
# ---------------------------------------------------------------------------
def test_native_autotracer_captures_events():
    tracer = xplainit.AutoTracer()

    def fib(n):
        if n < 2:
            return n
        return fib(n - 1) + fib(n - 2)

    tracer.start()
    result = fib(5)
    tracer.stop()

    assert result == 5, f"fib(5) should be 5, got {result!r}"

    events = _events_list(tracer.get_events())
    assert isinstance(events, list), "get_events() must return a list, not a raw string"

    # The old no-op captured ZERO events. A working settrace hook records many
    # enters for the recursive fib plus this test's own frame.
    fib_enters = _count_enters(events, "fib")
    assert fib_enters >= 5, (
        f"native AutoTracer captured no/too-few real events "
        f"(fib enters={fib_enters}); it is a no-op"
    )
    assert not tracer.is_active(), "tracer should be inactive after stop()"
    print(f"  [defect#1] native AutoTracer captured {len(events)} events "
          f"({fib_enters} fib enters) OK")


def test_native_autotracer_captures_exception():
    tracer = xplainit.AutoTracer()

    def boom():
        return 1 / 0

    tracer.start()
    try:
        boom()
    except ZeroDivisionError:
        pass
    tracer.stop()

    events = _events_list(tracer.get_events())
    assert _count_exceptions(events) >= 1, "native AutoTracer did not capture the exception"
    print("  [defect#1] native AutoTracer captured the ZeroDivisionError OK")


# ---------------------------------------------------------------------------
# Defect #2: auto_trace() must not raise and must enable working tracing
# ---------------------------------------------------------------------------
def test_auto_trace_enables_tracing():
    # This is the exact call that previously raised ModuleNotFoundError.
    backend = xplainit.auto_trace()
    try:
        def compute(a, b):
            return a * b + 1

        value = compute(6, 7)
    finally:
        xplainit.disable_auto_trace()

    assert value == 43, f"traced function returned wrong value: {value!r}"

    events = _events_list(backend.get_events())
    assert isinstance(events, (list,)), "backend.get_events() should be JSON-parsable"
    assert _count_enters(events, "compute") >= 1, (
        "auto_trace() did not actually capture the traced call"
    )
    print(f"  [defect#2] auto_trace() enabled tracing and captured "
          f"{len(events)} events OK")


def test_auto_trace_helper_package_importable():
    # Root cause of defect #2 was the missing xplainit.python submodule.
    import importlib

    mod = importlib.import_module("xplainit.python.tracer")
    assert hasattr(mod, "AutoTracer"), "xplainit.python.tracer.AutoTracer missing"
    print("  [defect#2] xplainit.python.tracer is importable from the package OK")


# ---------------------------------------------------------------------------
# Defect #3: @explain_function must return a callable that works AND records
# ---------------------------------------------------------------------------
def test_explain_function_returns_working_callable():
    @xplainit.explain_function
    def add(a, b):
        return a + b

    assert callable(add), "explain_function must return a callable, not None"
    assert add(2, 3) == 5, "decorated function must return the original result"
    print("  [defect#3] @explain_function returns a working callable OK")


def test_explain_function_records_trace():
    backend = xplainit.explain_backend()
    backend.clear()

    @xplainit.explain_function
    def multiply(x, y):
        return x * y

    assert multiply(4, 5) == 20

    events = _events_list(backend.get_events())
    assert _count_enters(events, "multiply") >= 1, (
        "@explain_function did not record a FunctionEnter (still a stub?)"
    )
    print(f"  [defect#3] @explain_function recorded {len(events)} events OK")


def test_packaged_helper_matches_source_tree():
    """Guard against drift between the canonical source-tree helper package
    (``xplainit-python/python``) and the copy packaged for the wheel
    (``xplainit-python/python_src/xplainit/python``)."""
    here = os.path.dirname(os.path.abspath(__file__))
    canonical = os.path.join(here, "python")
    packaged = os.path.join(here, "python_src", "xplainit", "python")
    if not os.path.isdir(packaged):
        # Running from an installed wheel (no source tree) - nothing to compare.
        print("  [packaging] source tree not present; skipping drift check")
        return
    for fname in ("tracer.py", "decorators.py", "__init__.py"):
        with open(os.path.join(canonical, fname), "rb") as fh:
            a = fh.read()
        with open(os.path.join(packaged, fname), "rb") as fh:
            b = fh.read()
        assert a == b, (
            f"packaged xplainit/python/{fname} has drifted from the canonical "
            f"python/{fname}; keep them in sync"
        )
    print("  [packaging] packaged helper matches source-tree canonical copy OK")


def test_explain_function_records_exception():
    backend = xplainit.explain_backend()
    backend.clear()

    @xplainit.explain_function
    def bad():
        return 1 / 0

    try:
        bad()
        raise AssertionError("expected ZeroDivisionError to propagate")
    except ZeroDivisionError:
        pass

    events = _events_list(backend.get_events())
    assert _count_exceptions(events) >= 1, "@explain_function did not record the exception"
    print("  [defect#3] @explain_function recorded the exception and re-raised OK")


def main():
    tests = [
        test_native_autotracer_captures_events,
        test_native_autotracer_captures_exception,
        test_auto_trace_enables_tracing,
        test_auto_trace_helper_package_importable,
        test_explain_function_returns_working_callable,
        test_explain_function_records_trace,
        test_explain_function_records_exception,
        test_packaged_helper_matches_source_tree,
    ]
    print("=" * 70)
    print("PUBLIC API TESTS (revert-sensitive)")
    print("=" * 70)
    failures = 0
    for test in tests:
        # sys.settrace-based tests must run with no tracer already installed.
        sys.settrace(None)
        try:
            test()
        except AssertionError as exc:
            failures += 1
            print(f"  FAIL: {test.__name__}: {exc}")
        except Exception as exc:  # noqa: BLE001
            failures += 1
            print(f"  ERROR: {test.__name__}: {type(exc).__name__}: {exc}")
        finally:
            sys.settrace(None)
    print("-" * 70)
    total = len(tests)
    print(f"{total - failures}/{total} passed")
    if failures:
        sys.exit(1)
    print("ALL PUBLIC API TESTS PASSED!")


if __name__ == "__main__":
    main()
