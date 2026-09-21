# Xplainit Python Bindings

**Natural language explanations for your Python code execution**

## Overview

Xplainit is a runtime code explanation framework that traces your Python code execution and generates human-readable explanations of what happened. It helps you understand, debug, and document your code by providing clear, step-by-step narratives of execution flow.

## Features

✅ **Natural Language Explanations** - Get human-readable descriptions of code execution  
✅ **Multiple Verbosity Levels** - Choose from brief, normal, detailed, or debug output  
✅ **Function Call Tracking** - See function calls, arguments, and return values  
✅ **Exception Handling** - Detailed error context and stack traces  
✅ **Recursive Function Support** - Track recursive calls with depth information  
✅ **Performance Profiling** - Measure execution time of functions  
✅ **Easy-to-use Decorators** - Simple `@trace` decorator for selective tracing  
✅ **Zero-overhead when disabled** - No performance impact when not tracing  

## Installation

### Prerequisites
- Python 3.8+
- Rust (for building from source)
- maturin (Python package builder)

### Build and Install

```bash
# Clone the repository
git clone https://github.com/Xplainit-Foundation/xplainit-framework.git
cd xplainit-framework/xplainit-python

# Create virtual environment
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install maturin
pip install maturin

# Build and install the module
maturin develop --release
```

## Quick Start

### Basic Usage

```python
from xplainit import Xplainit
from python.decorators import trace

# Create the backend
backend = Xplainit(enabled=True)

# Use the @trace decorator
@trace(backend=backend)
def add(a, b):
    return a + b

result = add(5, 3)

# Get explanations
explanations = backend.get_explanations()
print(explanations)
```

**Output:**
```
Calling function add with 2 argument(s)
Function add returned 8
```

### Verbosity Levels

```python
# Brief - minimal information
backend.get_explanations(verbosity="brief")
# Output: Calling add
#         add returned

# Normal - standard detail (default)
backend.get_explanations(verbosity="normal")
# Output: Calling function add with 2 argument(s)
#         Function add returned 8

# Detailed - comprehensive information
backend.get_explanations(verbosity="detailed")
# Output: Calling function add at /path/to/file.py:10
#           Arguments:
#             a: integer = 5
#             b: integer = 3
#         Function add returned 8 (integer) after 0ns

# Debug - maximum detail
backend.get_explanations(verbosity="debug")
# Output: Calling function add at /path/to/file.py:10
#           Arguments:
#             a: integer = 5
#             b: integer = 3
#         Function add completed and returned 8 (integer) (execution time: 0.000ms)
```

## Decorators

### @trace - Trace a function

```python
from python.decorators import trace

@trace(backend=backend)
def my_function(a, b):
    return a + b
```

### @trace_class - Trace all methods in a class

```python
from python.decorators import trace_class

@trace_class(backend=backend)
class Calculator:
    def add(self, a, b):
        return a + b
    
    def multiply(self, a, b):
        return a * b
```

### @trace_recursive - Trace recursive functions with depth

```python
from python.decorators import trace_recursive

@trace_recursive(backend=backend, max_depth=20)
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

result = fibonacci(10)
# Output shows: Calling fibonacci [depth=1], Calling fibonacci [depth=2], etc.
```

### @profile - Measure execution time

```python
from python.decorators import profile

@profile(backend=backend, name="slow_operation")
def slow_function():
    time.sleep(0.01)
    return "done"

# Output: [Profile] slow_operation: 10.19ms
```

## Backend Methods

### Core Methods

```python
# Create backend
backend = Xplainit(enabled=True, verbosity="detailed")

# Enable/disable tracing
backend.enable()
backend.disable()

# Clear captured events
backend.clear()

# Get events as JSON
events_json = backend.get_events()

# Get natural language explanations
explanations = backend.get_explanations(verbosity="normal")

# Print explanations to console
backend.print_explanations(verbosity="detailed")

# Set verbosity level
backend.set_verbosity("detailed")

# Get statistics
stats = backend.get_stats()

# Check if enabled
is_enabled = backend.is_enabled()
```

## Advanced Usage

### Nested Function Calls

```python
@trace(backend=backend)
def multiply(a, b):
    return a * b

@trace(backend=backend)
def calculate_area(length, width):
    return multiply(length, width)

@trace(backend=backend)
def calculate_volume(length, width, height):
    base = calculate_area(length, width)
    return multiply(base, height)

volume = calculate_volume(3, 4, 5)

# Explanations show the full call stack:
# Calling calculate_volume with 3 argument(s)
# Calling calculate_area with 2 argument(s)
# Calling multiply with 2 argument(s)
# Function multiply returned 12
# Function calculate_area returned 12
# Calling multiply with 2 argument(s)
# Function multiply returned 60
# Function calculate_volume returned 60
```

### Exception Handling

```python
@trace(backend=backend)
def divide(a, b):
    return a / b

try:
    result = divide(10, 0)
except ZeroDivisionError:
    pass

explanations = backend.get_explanations(verbosity="detailed")
# Output:
# Calling function divide at /path/to/file.py:10
#   Arguments:
#     a: integer = 10
#     b: integer = 0
# ❌ EXCEPTION (UNCAUGHT ZeroDivisionError)
#   Error: division by zero
#   Location: /path/to/file.py:10
```

### Complex Data Structures

```python
@trace(backend=backend)
def process_user(user_id, profile):
    return {
        'user_id': user_id,
        'name': profile['name'],
        'age': profile['age']
    }

user_data = {
    'name': 'Alice',
    'age': 30,
    'email': 'alice@example.com'
}

result = process_user(12345, user_data)

# Detailed output shows:
# Calling function process_user at /path/to/file.py:20
#   Arguments:
#     user_id: integer = 12345
#     profile: dict = {'name': 'Alice', 'age': 30, 'email': 'alice@example.com'}
# Function process_user returned {'user_id': 12345, 'name': 'Alice', 'age': 30}
```

## Real-World Use Cases

### 1. Debugging

```python
# Understand why a function returns unexpected results
@trace(backend=backend)
def buggy_function(data):
    # ... complex logic ...
    return result

# The explanation reveals step-by-step execution
explanations = backend.get_explanations(verbosity="detailed")
```

### 2. Learning Algorithms

```python
# See how a sorting algorithm works step-by-step
@trace(backend=backend)
def quicksort(arr):
    if len(arr) <= 1:
        return arr
    pivot = arr[len(arr) // 2]
    left = [x for x in arr if x < pivot]
    middle = [x for x in arr if x == pivot]
    right = [x for x in arr if x > pivot]
    return quicksort(left) + middle + quicksort(right)

# Watch the recursion unfold
result = quicksort([3, 6, 8, 10, 1, 2, 1])
```

### 3. Documentation

```python
# Generate execution traces for documentation
@trace(backend=backend)
def api_endpoint(request):
    # Process API request
    return response

# Run example request
response = api_endpoint(example_request)

# Export explanation as documentation
docs = backend.get_explanations(verbosity="detailed")
```

### 4. Testing

```python
# Verify function calls during tests
def test_user_creation():
    backend.clear()
    
    create_user("Alice", 30)
    
    events = json.loads(backend.get_events())
    assert len(events) >= 2  # Enter + exit
    assert any('create_user' in str(e) for e in events)
```

## Performance

The tracer is designed for minimal overhead:

- **Lock-free event storage** - Uses circular buffer for high performance
- **Selective tracing** - Only trace what you need with decorators
- **Configurable sampling** - Reduce overhead with sampling rate
- **Module filtering** - Exclude stdlib and third-party code

### Performance Tips

1. Use `@trace` decorator only on functions you need to trace
2. Set appropriate `max_depth` for recursive functions
3. Use `verbosity="brief"` for large-scale tracing
4. Clear events regularly with `backend.clear()` to free memory

## Testing

Run the test suite:

```bash
# Unit tests
cargo test -p xplainit-python

# Python tests
python test_decorators.py
python test_explanations_final.py

# Advanced demos
python demo_advanced.py
python demo_debugging.py
```

## Architecture

```
┌─────────────────────────────────────────────┐
│           Python Application                 │
│  (Your code with @trace decorators)         │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│        Python Decorators                     │
│  (trace, trace_class, profile, etc.)        │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│        Xplainit Backend (Rust)              │
│  • Event capture & storage                   │
│  • Explanation generation                    │
│  • Verbosity handling                        │
│  • Performance optimization                  │
└──────────────────┴──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│        Natural Language Output              │
│  "Calling function add with 2 argument(s)"  │
│  "Function add returned 8"                  │
└─────────────────────────────────────────────┘
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

## Links

- [GitHub Repository](https://github.com/Xplainit-Foundation/xplainit-framework)
- [Documentation](https://github.com/Xplainit-Foundation/xplainit-framework#readme)
- [Examples](./examples/)
- [API Reference](./docs/api.md)

## Support

- 📧 Email: support@xplainit.dev
- 💬 Discord: [Join our community](https://discord.gg/xplainit)
- 🐛 Issues: [GitHub Issues](https://github.com/Xplainit-Foundation/xplainit-framework/issues)

---

**Made with ❤️ by the Xplainit Team**
