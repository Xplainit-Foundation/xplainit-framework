# Xplainit Python Bindings - Explanation Generation Demo

This document demonstrates the natural language explanation generation capabilities of the Xplainit Python bindings.

## Overview

The Xplainit framework can automatically trace Python code execution and generate human-readable explanations of what happened, including:

- Function calls and returns
- Argument values
- Return values
- Exceptions and errors
- Recursive call depth
- Execution context

## Installation

```bash
# Build and install the Python module
cd xplainit-python
python -m venv .venv
source .venv/bin/activate
pip install maturin
maturin develop --release
```

## Basic Usage

### 1. Simple Function Tracing with Decorators

```python
from xplainit import Xplainit
from python.decorators import trace

# Create the backend
backend = Xplainit(enabled=True, verbosity="detailed")

# Trace a function with the @trace decorator
@trace(backend=backend)
def add(a, b):
    return a + b

result = add(5, 3)

# Get natural language explanations
explanations = backend.get_explanations(verbosity="normal")
print(explanations)
```

**Output:**
```
Calling function add with 2 argument(s)
Function add returned 8
```

### 2. Recursive Function Tracing

```python
from python.decorators import trace_recursive

backend.clear()

@trace_recursive(backend=backend, max_depth=5)
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

result = factorial(4)

# Get detailed explanations
backend.set_verbosity("detailed")
explanations = backend.get_explanations(verbosity="detailed")
print(explanations)
```

**Output:**
```
Calling function factorial [depth=1] at /path/to/file.py:50
  Arguments:
    depth: integer = 1
Calling function factorial [depth=2] at /path/to/file.py:50
  Arguments:
    depth: integer = 2
Calling function factorial [depth=3] at /path/to/file.py:50
  Arguments:
    depth: integer = 3
Calling function factorial [depth=4] at /path/to/file.py:50
  Arguments:
    depth: integer = 4
```

### 3. Error Handling and Exception Tracking

```python
@trace(backend=backend)
def divide(a, b):
    return a / b

try:
    result = divide(10, 0)
except ZeroDivisionError as e:
    print(f"Caught error: {e}")

# Get error explanation
explanations = backend.get_explanations(verbosity="detailed")
print(explanations)
```

**Output:**
```
Calling function divide at /path/to/file.py:78
  Arguments:
    b: integer = 0
    a: integer = 10
❌ EXCEPTION (UNCAUGHT ZeroDivisionError)
  Error: division by zero
  Location: /path/to/file.py:78
```

## Verbosity Levels

The explanation generator supports multiple verbosity levels:

### Brief
- Minimal information
- Just the essential facts

### Normal
- Standard amount of detail
- Function name, arguments, return values

### Detailed
- Comprehensive information
- Includes file locations, line numbers, full argument details

### Debug
- Maximum detail
- All available information

## Decorator Options

### @trace
```python
@trace(backend=backend)
def my_function(a, b):
    return a + b
```

### @trace_class
```python
@trace_class(backend=backend)
class MyClass:
    def method1(self):
        pass
    
    def method2(self):
        pass
```

### @profile
```python
@profile(backend=backend, name="slow_operation")
def slow_function():
    time.sleep(0.01)
    return "done"
```

Output: `[Profile] slow_operation: 10.37ms`

### @trace_recursive
```python
@trace_recursive(backend=backend, max_depth=10)
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)
```

## Backend Methods

### Core Methods
- `enable()` - Enable tracing
- `disable()` - Disable tracing
- `clear()` - Clear all captured events
- `get_events()` - Get events as JSON string
- `get_explanations(verbosity)` - Get natural language explanations
- `print_explanations(verbosity)` - Print explanations to stdout
- `set_verbosity(level)` - Set verbosity level
- `get_stats()` - Get statistics about captured events

### Callback Methods (used by decorators)
- `on_function_enter(name, args, filename, line)`
- `on_function_exit(name, return_value, filename, line)`
- `on_exception(exc_type, exc_message, filename, line)`

## Use Cases

1. **Debugging**: Understand what your code is doing during execution
2. **Learning**: See how algorithms execute step-by-step
3. **Documentation**: Generate execution traces for documentation
4. **Testing**: Verify function calls and argument values
5. **Profiling**: Measure execution time with @profile decorator
6. **Error Analysis**: Get detailed error context and stack traces

## Performance

The tracer is designed for minimal overhead:
- Events are stored in a lock-free circular buffer
- Sampling rate can be adjusted to reduce overhead
- Selective tracing using decorators avoids tracing entire codebase
- Module inclusion/exclusion patterns for fine-grained control

## Testing

Run the test suite:

```bash
cd xplainit-python
python test_decorators.py
python test_explanations_final.py
```

All tests should pass with events captured and explanations generated correctly.
