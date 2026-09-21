# Python

Python is the most complete tracing surface. The native module is built with
maturin and imported as `xplainit`; a Python helper package provides an
automatic tracer and decorators.

## Build

```sh
source .venv/bin/activate
cd xplainit-python
maturin develop --release
```

## Native module API (`import xplainit`)

The compiled module exposes:

- Classes: `Xplainit`, `XplainitContext`, `AutoTracer`.
- Functions: `enable()`, `disable()`, `is_enabled()`, `explain_function()`,
  `get_last_explanation()`, `auto_trace()`.

`Xplainit` methods include `enable()`, `disable()`, `is_enabled()`,
`get_events()`, `get_last_explanation()`, `clear()`, `set_verbosity(level)`,
`get_stats()`, `get_explanations(verbosity=None)`,
`print_explanations(verbosity=None)`, and manual event hooks
(`on_function_enter`, `on_function_exit`, `on_exception`, `on_line_execute`).

`XplainitContext` supports the `with` statement (`__enter__`/`__exit__`).

```python
import xplainit

xplainit.enable()
# ... run code ...
xplainit.disable()
print(xplainit.get_last_explanation())
```

## Automatic tracing (`sys.settrace`)

The Python helper package (`xplainit-python/python`) provides `XplainitTracer`
(the `AutoTracer`), which installs a `sys.settrace()` hook on `start()` and
removes it on `stop()`. It captures call/return/exception events and applies a
frame filter that traces user / `__main__` code while excluding Xplainit's own
internals and genuine stdlib.

```python
from xplainit import XplainitTracer  # via the helper package

with XplainitTracer() as tracer:
    run_my_code()
events = tracer.get_events()
```

The automatic tracer is verified by `test_automatic_tracing.py` (3/3),
`test_frame_filter.py` (11/11), and `test_depth_accounting.py` (23/23).

## Decorators

For hand-picked functions without a global hook:

- `@trace(backend=None, capture_args=True, capture_return=True, capture_locals=False)`
- `@trace_class(backend=None, exclude_methods=None)`
- `@profile(backend=None, name=None)`
- `@trace_recursive(backend=None, max_depth=100)`

```python
from xplainit import trace

@trace()
def compute(x):
    return x * 2
```

Verified by `test_decorators.py`.

## Honest overhead note

Full automatic tracing installs a global `sys.settrace()` hook, so a Python
callback fires on **every** call/return. Measured overhead on a call-heavy
microbenchmark (`fib(24)`) is roughly **~1200x** — the `<10%` goal is **not
achievable for full tracing**. Sampling barely helps because the per-event
Python callback still fires. For low overhead, trace **selectively**: use the
`@trace` decorators on specific functions, narrow `include_modules`, use a
shallow `max_depth`, and keep line tracing off (the default). See
[performance tuning](../advanced/performance-tuning.md) and
[`../../PERFORMANCE.md`](../../PERFORMANCE.md).
