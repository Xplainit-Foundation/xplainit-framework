# Xplainit Python

Python bindings for the Xplainit Framework - Understand what your code is doing in plain English.

## Features

- 🔍 **Automatic Runtime Tracing** - See every function call, variable assignment, and control flow decision
- 💬 **Plain English Explanations** - Convert runtime events into human-readable descriptions
- 🐛 **Error Explanation** - Get detailed explanations of what went wrong and how to fix it
- ⚡ **Zero Overhead When Disabled** - No performance impact when tracing is off
- 🎯 **Selective Tracing** - Use decorators to trace specific functions
- 📊 **Multiple Verbosity Levels** - From brief to debug-level detail

## Installation

```bash
pip install xplainit
```

Or from source:

```bash
git clone https://github.com/xplainit/xplainit.git
cd xplainit/xplainit-python
pip install maturin
maturin develop
```

## Quick Start

### Global Tracing

```python
import xplainit

# Enable tracing
xplainit.enable()

def calculate(x, y):
    result = x + y
    return result

value = calculate(10, 20)
# Output: Calling function calculate with 2 argument(s)
#         Variable result was assigned the value 30
#         Function calculate returned 30

xplainit.disable()
```

### Context Manager (Scoped Tracing)

```python
import xplainit

def factorial(n):
    if n <= 0:
        return 1
    return n * factorial(n - 1)

with xplainit.XplainitContext(enabled=True, verbosity="normal"):
    result = factorial(5)
    # Only this block is traced

print(f"Result: {result}")
```

### Decorator-Based Tracing

```python
import xplainit

@xplainit.explain_function
def divide(a, b):
    return a / b

# This function will be traced
result = divide(10, 2)

# Other functions won't be traced unless decorated
```

### Error Explanation

```python
import xplainit

xplainit.enable()

try:
    items = [1, 2, 3]
    value = items[10]  # Index out of bounds
except IndexError:
    explanation = xplainit.get_last_explanation()
    print(explanation)
    # Output: ❌ INDEX OUT OF BOUNDS ERROR
    #         Tried to access index 10 in collection but it only has 3 elements
    #         Valid indices: 0 to 2
    #         💡 Fix: Check that your index is within the valid range

xplainit.disable()
```

## Advanced Usage

### Custom Configuration

```python
import xplainit

explainer = xplainit.Xplainit(
    enabled=True,
    verbosity="detailed",  # brief, normal, detailed, debug
    output="stdout"        # stdout, stderr, or file path
)

explainer.start()

# Your code here

explainer.stop()
```

### Verbosity Levels

- **brief**: Minimal output (one line per event)
- **normal**: Balanced output with key information
- **detailed**: Full context including values and types
- **debug**: Everything including framework internals

### Environment Variables

```bash
# Enable/disable tracing
export XPLAINIT_ENABLED=true

# Set verbosity level
export XPLAINIT_VERBOSITY=detailed

# Set output destination
export XPLAINIT_OUTPUT=stderr

# Rate limiting (events per second)
export XPLAINIT_MAX_EVENTS_PER_SEC=1000
```

## Examples

See the `examples/` directory for complete working examples:

- `basic_usage.py` - Introduction to core features
- `decorator_usage.py` - Selective tracing with decorators
- `error_handling.py` - Error explanation demonstrations

## API Reference

### Module Functions

- `xplainit.enable()` - Enable global tracing
- `xplainit.disable()` - Disable global tracing
- `xplainit.is_enabled()` - Check if tracing is enabled
- `xplainit.get_last_explanation()` - Get the most recent explanation

### Classes

#### `Xplainit(enabled=True, verbosity="normal", output="stdout")`

Main tracing class.

**Methods:**
- `start()` - Start tracing (installs sys.settrace)
- `stop()` - Stop tracing
- `enable()` - Enable tracing
- `disable()` - Disable tracing
- `is_enabled()` - Check enabled status
- `get_events()` - Get captured events as JSON
- `get_last_explanation()` - Get last explanation
- `clear()` - Clear all captured events
- `set_verbosity(level)` - Change verbosity level
- `get_stats()` - Get tracing statistics

#### `XplainitContext(enabled=True, verbosity="normal")`

Context manager for scoped tracing.

**Usage:**
```python
with xplainit.XplainitContext(enabled=True):
    # Code in this block will be traced
    pass
```

### Decorators

#### `@xplainit.explain_function`

Decorate a function to enable tracing for that function only.

```python
@xplainit.explain_function
def my_function():
    pass
```

## Performance

- **Enabled**: ~5-15% overhead depending on verbosity and event frequency
- **Disabled**: Zero overhead (atomic boolean check, optimized out by compiler)
- **Selective tracing**: Only decorated functions incur overhead

## Python Version Support

- Python 3.8+
- Python 3.12+ (sys.monitoring support coming soon)

## Platform Support

- Windows
- macOS
- Linux

## Contributing

Contributions welcome! See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT License ([LICENSE-MIT](../LICENSE-MIT))

at your option.

## Links

- [GitHub Repository](https://github.com/xplainit/xplainit)
- [Documentation](https://xplainit.readthedocs.io)
- [Issue Tracker](https://github.com/xplainit/xplainit/issues)
