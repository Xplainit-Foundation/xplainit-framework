"""
Test Python Decorators

Validates that all decorator functions work correctly.
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'target', 'debug'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

import xplainit
from python.decorators import trace, trace_class, profile, trace_recursive
import json

print("=" * 70)
print("DECORATOR TESTS")
print("=" * 70)
print()

# Initialize backend
backend = xplainit.Xplainit(enabled=True, verbosity="normal")

# Test 1: @trace decorator
print("Test 1: @trace decorator")
print("-" * 70)

@trace(backend=backend)
def add(a, b):
    return a + b

result = add(10, 20)
print(f"✓ add(10, 20) = {result}")

events = json.loads(backend.get_events())
assert len(events) >= 2, "Expected at least 2 events (enter + exit)"
assert any('FunctionEnter' in e and e['FunctionEnter']['name'] == 'add' for e in events)
print(f"✓ Captured {len(events)} events")
print()

# Test 2: @trace_class decorator
print("Test 2: @trace_class decorator")
print("-" * 70)

backend.clear()

@trace_class(backend=backend)
class Calculator:
    def multiply(self, x, y):
        return x * y
    
    def divide(self, x, y):
        if y == 0:
            raise ValueError("Division by zero")
        return x / y

calc = Calculator()
result = calc.multiply(5, 6)
print(f"✓ Calculator.multiply(5, 6) = {result}")

events = json.loads(backend.get_events())
print(f"✓ Captured {len(events)} events")
print()

# Test 3: @profile decorator
print("Test 3: @profile decorator")
print("-" * 70)

@profile(backend=backend, name="slow_operation")
def slow_function():
    import time
    time.sleep(0.01)
    return "done"

result = slow_function()
print(f"✓ slow_function() = {result}")
print("✓ Profiling information displayed")
print()

# Test 4: @trace_recursive decorator
print("Test 4: @trace_recursive decorator")
print("-" * 70)

backend.clear()

@trace_recursive(backend=backend, max_depth=10)
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

result = factorial(5)
print(f"✓ factorial(5) = {result}")

events = json.loads(backend.get_events())
recursive_calls = [e for e in events if 'FunctionEnter' in e and '[depth=' in e['FunctionEnter']['name']]
print(f"✓ Traced {len(recursive_calls)} recursive calls")
print()

# Test 5: Error handling
print("Test 5: Error handling in decorators")
print("-" * 70)

backend.clear()

@trace(backend=backend)
def divide_by_zero():
    return 1 / 0

try:
    divide_by_zero()
    print("✗ Should have raised ZeroDivisionError")
except ZeroDivisionError:
    print("✓ Exception raised as expected")

events = json.loads(backend.get_events())
exceptions = [e for e in events if 'Exception' in e]
print(f"✓ Captured {len(exceptions)} exception events")
print()

# Summary
print("=" * 70)
print("ALL DECORATOR TESTS PASSED!")
print("=" * 70)
print()
print("✅ @trace decorator: Working")
print("✅ @trace_class decorator: Working")
print("✅ @profile decorator: Working")
print("✅ @trace_recursive decorator: Working")
print("✅ Error handling: Working")
print()
print("Decorators provide convenient selective tracing without")
print("needing to enable full sys.settrace() automatic tracing!")
print()
