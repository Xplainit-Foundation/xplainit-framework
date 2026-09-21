#!/usr/bin/env python3
"""
Direct unit tests for the AutoTracer frame filter.

These tests exercise the Python `_should_trace` / `_is_stdlib_file` logic that
was the actual FEAT-002 bug, WITHOUT relying on end-to-end event counts. They
pin the intended contract so the fix cannot be silently reverted:

  (a) a user / __main__ frame IS traced,
  (b) a stdlib file path IS excluded,
  (c) the tracer's own __file__ IS excluded,
  (d) a user module whose *name* contains the substring "tracer" is STILL traced.

If the FEAT-002 fix were reverted (i.e. '__main__' re-added to the stdlib
exclusion, or the substring self-exclusion `'tracer' in module_name` restored),
these assertions FAIL.

Runnable as a standalone script (consistent with the repo's script-style tests):

    python xplainit-python/test_frame_filter.py
"""

import sys
import os
import json
import tempfile
import textwrap
import importlib.util
import types

# Make the tracer module importable regardless of the working directory.
_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(_HERE, 'python'))

from tracer import AutoTracer  # noqa: E402


class _FakeCode:
    """Minimal stand-in for a code object exposing what _should_trace reads."""

    def __init__(self, co_filename, co_name='fake_func'):
        self.co_filename = co_filename
        self.co_name = co_name


class _FakeFrame:
    """Minimal stand-in for a frame object exposing what _should_trace reads."""

    def __init__(self, filename, module_name):
        self.f_code = _FakeCode(filename)
        self.f_globals = {'__name__': module_name}
        self.f_lineno = 1
        self.f_locals = {}


def _new_tracer():
    """Create a tracer instance with a dummy backend (never actually called)."""
    return AutoTracer(backend=object(), max_depth=100)


# ---------------------------------------------------------------------------
# Helpers to obtain a REAL frame from a REAL user file (so the test is
# meaningful, not just a construction of fake attributes).
# ---------------------------------------------------------------------------

def _make_real_user_frame(module_name):
    """
    Write a tiny module to a temp file, import it under `module_name`, and
    capture a real frame object created while executing that module's code.
    Returns (frame, filename).
    """
    src = textwrap.dedent(
        """
        import sys

        def capture_frame():
            # sys._getframe() returns the frame currently executing this
            # user-defined function inside the temp (user) file.
            return sys._getframe()
        """
    )
    tmp = tempfile.NamedTemporaryFile(
        mode='w', suffix='.py', delete=False, prefix='xplainit_ff_'
    )
    tmp.write(src)
    tmp.close()

    spec = importlib.util.spec_from_file_location(module_name, tmp.name)
    mod = importlib.util.module_from_spec(spec)
    # Force the module's recorded name so f_globals['__name__'] matches what a
    # real user import/run would produce.
    mod.__name__ = module_name
    sys.modules[module_name] = mod
    spec.loader.exec_module(mod)
    mod.__name__ = module_name

    frame = mod.capture_frame()
    return frame, tmp.name


def _cleanup(filename, module_name):
    sys.modules.pop(module_name, None)
    try:
        os.unlink(filename)
    except OSError:
        pass


results = []


def check(label, condition):
    status = "PASS" if condition else "FAIL"
    print(f"  [{status}] {label}")
    results.append((label, bool(condition)))


print("=" * 70)
print("FRAME FILTER UNIT TESTS (_should_trace / _is_stdlib_file)")
print("=" * 70)


# ---------------------------------------------------------------------------
# (a) A user __main__ frame IS traced.
#     Uses this very script's real frame (module __name__ == '__main__').
# ---------------------------------------------------------------------------
print("\n(a) __main__ / user frame is traced")
tracer = _new_tracer()

main_frame = sys._getframe()  # running as __main__ from a real file on disk
check(
    "current __main__ frame is traced (_should_trace True)",
    tracer._should_trace(main_frame) is True,
)
check(
    "__main__ is NOT classified as stdlib",
    tracer._is_stdlib_file(main_frame.f_code.co_filename, '__main__') is False,
)

# Also exercise a real frame from a freshly-imported user module.
user_frame, user_file = _make_real_user_frame('some_user_module')
try:
    check(
        "real frame from a user module file is traced",
        tracer._should_trace(user_frame) is True,
    )
    check(
        "user module file is NOT classified as stdlib",
        tracer._is_stdlib_file(user_file, 'some_user_module') is False,
    )
finally:
    _cleanup(user_file, 'some_user_module')


# ---------------------------------------------------------------------------
# (b) A stdlib file path IS excluded.
#     Use a real stdlib module's file (os) to avoid brittle synthetic paths.
# ---------------------------------------------------------------------------
print("\n(b) stdlib file path is excluded")
tracer = _new_tracer()

stdlib_file = os.path.realpath(os.__file__)  # e.g. .../lib/python3.9/os.py
check(
    "real stdlib file (os.py) IS classified as stdlib",
    tracer._is_stdlib_file(stdlib_file, 'os') is True,
)
stdlib_frame = _FakeFrame(stdlib_file, 'os')
check(
    "frame in a stdlib file is NOT traced",
    tracer._should_trace(stdlib_frame) is False,
)


# ---------------------------------------------------------------------------
# (c) The tracer's OWN __file__ IS excluded (prevents infinite recursion).
# ---------------------------------------------------------------------------
print("\n(c) tracer's own source file is excluded")
tracer = _new_tracer()

self_file = tracer._self_filename
check("tracer knows its own filename", bool(self_file))
self_frame = _FakeFrame(self_file, 'tracer')
check(
    "frame in the tracer's own file is NOT traced",
    tracer._should_trace(self_frame) is False,
)


# ---------------------------------------------------------------------------
# (d) A user module whose NAME contains the substring "tracer" is STILL traced.
#     This is the exact regression the substring self-exclusion caused.
# ---------------------------------------------------------------------------
print("\n(d) user module named like '*tracer*' is still traced")
tracer = _new_tracer()

frame_named_tracer, file_named_tracer = _make_real_user_frame('request_tracer')
try:
    check(
        "user module 'request_tracer' is NOT classified as stdlib",
        tracer._is_stdlib_file(file_named_tracer, 'request_tracer') is False,
    )
    check(
        "user module 'request_tracer' IS traced",
        tracer._should_trace(frame_named_tracer) is True,
    )
finally:
    _cleanup(file_named_tracer, 'request_tracer')

# A user module literally named 'xplainit_tracer' (contains both loaded
# substrings) must still be user code - only exact 'xplainit'/'xplainit.'
# and the tracer's own file are excluded.
tracer = _new_tracer()
frame_x, file_x = _make_real_user_frame('xplainit_tracer_helpers')
try:
    check(
        "user module 'xplainit_tracer_helpers' IS traced",
        tracer._should_trace(frame_x) is True,
    )
finally:
    _cleanup(file_x, 'xplainit_tracer_helpers')


# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
print("\n" + "=" * 70)
passed = sum(1 for _, ok in results if ok)
total = len(results)
print(f"Frame filter tests: {passed}/{total} passed")
print("=" * 70)

if passed != total:
    failed = [label for label, ok in results if not ok]
    print("\nFAILED CHECKS:")
    for label in failed:
        print(f"  - {label}")
    sys.exit(1)

print("\n\u2713 ALL FRAME FILTER TESTS PASSED")
