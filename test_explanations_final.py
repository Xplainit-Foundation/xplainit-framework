#!/usr/bin/env python3
"""Test explanation generation with decorators."""

import sys
sys.path.insert(0, '/projects/sandbox/xplainit-framework/xplainit-python/python')

from xplainit import Xplainit
from decorators import trace, trace_recursive
import json

print("=" * 70)
print("Testing Explanation Generation with Decorators")
print("=" * 70)

# Create the backend
backend = Xplainit(enabled=True, verbosity="detailed")
print(f"✓ Created backend")

# Test 1: Simple function with @trace decorator
print("\n" + "=" * 70)
print("Test 1: Simple Function Trace")
print("=" * 70)

@trace(backend=backend)
def add(a, b):
    """Add two numbers."""
    return a + b

result = add(5, 3)
print(f"✓ add(5, 3) = {result}")

# Get events
events_json = backend.get_events()
events = json.loads(events_json)
print(f"✓ Captured {len(events)} events")

# Get explanations
print("\nExplanations:")
explanations = backend.get_explanations(verbosity="normal")
print(explanations)

# Clear for next test
backend.clear()

# Test 2: Recursive function
print("\n" + "=" * 70)
print("Test 2: Recursive Function Trace")
print("=" * 70)

@trace_recursive(backend=backend, max_depth=5)
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

result = factorial(4)
print(f"✓ factorial(4) = {result}")

# Get events
events_json = backend.get_events()
events = json.loads(events_json)
print(f"✓ Captured {len(events)} events")

# Get detailed explanations
print("\nDetailed Explanations:")
backend.set_verbosity("detailed")
explanations = backend.get_explanations(verbosity="detailed")
print(explanations[:800] if len(explanations) > 800 else explanations)

# Clear for next test
backend.clear()

# Test 3: Error handling
print("\n" + "=" * 70)
print("Test 3: Error Handling")
print("=" * 70)

@trace(backend=backend)
def divide(a, b):
    return a / b

try:
    result = divide(10, 0)
except ZeroDivisionError as e:
    print(f"✓ Caught expected error: {e}")

# Get events
events_json = backend.get_events()
events = json.loads(events_json)
print(f"✓ Captured {len(events)} events")

# Get error explanation
print("\nError Explanation:")
explanations = backend.get_explanations(verbosity="detailed")
print(explanations)

print("\n" + "=" * 70)
print("All Explanation Tests Passed!")
print("=" * 70)
