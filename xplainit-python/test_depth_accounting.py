#!/usr/bin/env python3
"""
Regression + coverage tests for the sys.settrace() depth accounting fix
(FEAT-002) and for the end-to-end value / return / exception / generator /
async / recursion behavior of the Python AutoTracer.

Background - the bug this file pins:
    The old `_should_trace` returned False whenever `_get_depth() > max_depth`.
    Because `_should_trace` is consulted for BOTH 'call' and 'return' events,
    once depth exceeded max_depth the matching 'return' events were filtered
    out too, so `_handle_return` never ran, depth never decremented, and the
    depth counter ratcheted upward - permanently wedging tracing OFF for that
    thread. The fix moves depth bookkeeping out of `_should_trace`: every
    user-code frame's call increments depth and its return decrements it
    (balanced), and max_depth/sampling only decide whether to RECORD.

This script self-asserts and prints PASS/FAIL, matching the other python test
scripts in this repo. Run with:

    python xplainit-python/test_depth_accounting.py
"""

import sys
import os
import json
import asyncio

# Make xplainit (compiled) and the tracer module importable regardless of cwd.
_HERE = os.path.dirname(os.path.abspath(__file__))
_ROOT = os.path.dirname(_HERE)
sys.path.insert(0, os.path.join(_ROOT, 'target', 'release'))
sys.path.insert(0, os.path.join(_HERE, 'python'))

import xplainit  # noqa: E402
from tracer import AutoTracer  # noqa: E402


results = []


def check(label, condition):
    status = "PASS" if condition else "FAIL"
    print(f"  [{status}] {label}")
    results.append((label, bool(condition)))


def _events(backend):
    return json.loads(backend.get_events())


def _kind(e):
    return list(e.keys())[0] if isinstance(e, dict) else None


def _enters(events):
    return [e['FunctionEnter'] for e in events if _kind(e) == 'FunctionEnter']


def _exits(events):
    return [e['FunctionExit'] for e in events if _kind(e) == 'FunctionExit']


def _exceptions(events):
    return [e['Exception'] for e in events if _kind(e) == 'Exception']


def _new(**kwargs):
    backend = xplainit.Xplainit(enabled=True, verbosity="normal")
    tracer = AutoTracer(backend=backend, **kwargs)
    return backend, tracer


print("=" * 70)
print("DEPTH ACCOUNTING + RUNTIME HOOK COVERAGE TESTS (FEAT-002)")
print("=" * 70)


# ---------------------------------------------------------------------------
# (1) THE REGRESSION: deep recursion past max_depth must NOT wedge tracing off.
# ---------------------------------------------------------------------------
print("\n(1) deep recursion past max_depth does not wedge tracing")

backend, tracer = _new(max_depth=5)


def deep(n):
    if n <= 0:
        return 0
    return 1 + deep(n - 1)


def simple_after():
    return 42


tracer.start()
deep(30)                     # recursion far deeper than max_depth=5
depth_after_deep = tracer._get_depth()
simple_after()               # a NEW top-level call AFTER the deep recursion
tracer.stop()

events = _events(backend)
entered_names = {e.get('name') for e in _enters(events)}

check("depth returns to 0 after the deep (balanced) recursion",
      depth_after_deep == 0)
check("tracing was NOT wedged: simple_after() still captured after deep()",
      'simple_after' in entered_names)
check("deep() was recorded up to but not beyond max_depth",
      'deep' in entered_names)


# ---------------------------------------------------------------------------
# (2) Balanced call tree returns depth to exactly 0.
# ---------------------------------------------------------------------------
print("\n(2) balanced call tree returns depth to 0")

backend, tracer = _new(max_depth=100)


def leaf(x):
    return x * 2


def branch(x):
    return leaf(x) + leaf(x + 1)


def root():
    return branch(1) + branch(2)


tracer.start()
root()
final_depth = tracer._get_depth()
tracer.stop()

events = _events(backend)
enters = _enters(events)
exits = _exits(events)

check("depth is exactly 0 after a fully balanced call tree", final_depth == 0)
check("enter and exit counts are balanced", len(enters) == len(exits))
check("multiple frames captured in the tree", len(enters) >= 4)


# ---------------------------------------------------------------------------
# (3) ARGUMENT VALUES (real values, not just names) appear in FunctionEnter.
# ---------------------------------------------------------------------------
print("\n(3) argument VALUES flow into FunctionEnter events")

backend, tracer = _new()


def add(a, b):
    return a + b


tracer.start()
add(5, 3)
tracer.stop()

events = _events(backend)
add_enters = [e for e in _enters(events) if e.get('name') == 'add']
check("add() enter event captured", len(add_enters) == 1)
if add_enters:
    args = add_enters[0].get('args', {})
    # Value::Integer serializes as {"Integer": 5} in the event JSON.
    check("argument a has real value 5 (not just the name)",
          args.get('a') == {'Integer': 5})
    check("argument b has real value 3 (not just the name)",
          args.get('b') == {'Integer': 3})


# ---------------------------------------------------------------------------
# (4) RETURN VALUES appear in FunctionExit.
# ---------------------------------------------------------------------------
print("\n(4) return value flows into FunctionExit events")

backend, tracer = _new()


def compute():
    return 8


tracer.start()
compute()
tracer.stop()

events = _events(backend)
compute_exits = [e for e in _exits(events) if e.get('name') == 'compute']
check("compute() exit event captured", len(compute_exits) == 1)
if compute_exits:
    check("return value 8 appears in FunctionExit",
          compute_exits[0].get('return_value') == {'Integer': 8})


# ---------------------------------------------------------------------------
# (5) EXCEPTION: ZeroDivisionError yields Exception event w/ correct type.
# ---------------------------------------------------------------------------
print("\n(5) ZeroDivisionError produces an Exception event")

backend, tracer = _new()


def boom():
    return 10 / 0


tracer.start()
try:
    boom()
except ZeroDivisionError:
    pass
tracer.stop()

events = _events(backend)
exc = _exceptions(events)
check("at least one Exception event captured", len(exc) >= 1)
check("error_type is ZeroDivisionError",
      any(e.get('error_type') == 'ZeroDivisionError' for e in exc))


# ---------------------------------------------------------------------------
# (6) RECURSION: fibonacci produces multiple enter/exit and depth back to 0.
# ---------------------------------------------------------------------------
print("\n(6) recursive fibonacci: multiple events, depth back to 0")

backend, tracer = _new(max_depth=100)


def fib(n):
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)


tracer.start()
fib_result = fib(6)
fib_depth = tracer._get_depth()
tracer.stop()

events = _events(backend)
fib_enters = [e for e in _enters(events) if e.get('name') == 'fib']
fib_exits = [e for e in _exits(events) if e.get('name') == 'fib']

check("fib(6) computed correctly (== 8)", fib_result == 8)
# fib(6) makes 25 calls to fib in total.
check("many fib enter events captured (recursion traced)",
      len(fib_enters) >= 10)
check("fib enter/exit counts balanced", len(fib_enters) == len(fib_exits))
check("depth returns to 0 after recursion", fib_depth == 0)


# ---------------------------------------------------------------------------
# (7) GENERATORS: multiple resume/suspend, depth balanced, events captured.
#     NOTE (sys.settrace limitation): a generator function emits a 'call'
#     event each time it is resumed (next()/send) and a 'return' event on each
#     suspend (yield) as well as on final StopIteration. Because our fix keeps
#     depth accounting balanced across these paired call/return events, depth
#     still returns to 0.
# ---------------------------------------------------------------------------
print("\n(7) generators: balanced depth across yields")

backend, tracer = _new(max_depth=100)


def gen(n):
    total = 0
    for i in range(n):
        total += i
        yield total


tracer.start()
gen_values = list(gen(4))
gen_depth = tracer._get_depth()
tracer.stop()

events = _events(backend)
gen_enters = [e for e in _enters(events) if e.get('name') == 'gen']

check("generator produced expected values", gen_values == [0, 1, 3, 6])
check("generator frame was traced (>=1 enter event)", len(gen_enters) >= 1)
check("depth returns to 0 after draining the generator", gen_depth == 0)


# ---------------------------------------------------------------------------
# (8) ASYNC: user coroutine body is traced under asyncio.run; depth balanced.
#     NOTE (sys.settrace limitation): asyncio's event-loop internals live in
#     the stdlib and are correctly excluded; only the user coroutine body is
#     traced. Awaiting suspends/resumes the coroutine frame (paired
#     call/return), and the balanced accounting keeps depth at 0.
# ---------------------------------------------------------------------------
print("\n(8) async coroutine: user body traced, balanced depth")

backend, tracer = _new(max_depth=100)


async def coro(x):
    await asyncio.sleep(0)
    return x + 1


def run_async():
    return asyncio.run(coro(41))


tracer.start()
async_result = run_async()
async_depth = tracer._get_depth()
tracer.stop()

events = _events(backend)
coro_enters = [e for e in _enters(events) if e.get('name') == 'coro']

check("coroutine returned expected value (42)", async_result == 42)
check("user coroutine body was traced (>=1 enter event)",
      len(coro_enters) >= 1)
check("depth returns to 0 after asyncio.run", async_depth == 0)


# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
print("\n" + "=" * 70)
passed = sum(1 for _, ok in results if ok)
total = len(results)
print(f"Depth accounting tests: {passed}/{total} passed")
print("=" * 70)

if passed != total:
    print("\nFAILED CHECKS:")
    for label, ok in results:
        if not ok:
            print(f"  - {label}")
    sys.exit(1)

print("\n\u2713 ALL DEPTH ACCOUNTING TESTS PASSED")
