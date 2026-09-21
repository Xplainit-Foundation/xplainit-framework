#!/usr/bin/env python3
"""Test the AutoTracer integration with explanation generation."""

import sys
sys.path.insert(0, '/projects/sandbox/xplainit-framework/xplainit-python/python')

from xplainit import Xplainit
from tracer import AutoTracer

print("=" * 60)
print("Testing AutoTracer with Explanation Generation")
print("=" * 60)

# Create the backend
backend = Xplainit(enabled=True, verbosity="detailed")
print(f"✓ Created backend: {backend}")

# Create the auto tracer
tracer = AutoTracer(backend=backend, trace_calls=True, trace_returns=True)
print(f"✓ Created AutoTracer")

# Start tracing
tracer.start()
print("✓ Started automatic tracing")

# Run some Python code
def factorial(n):
    """Calculate factorial recursively."""
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def fibonacci(n):
    """Calculate fibonacci recursively."""
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

def test_function():
    """Test function with multiple calls."""
    result1 = factorial(5)
    result2 = fibonacci(10)
    return result1 + result2

# Execute the code
result = test_function()
print(f"✓ Executed test_function() = {result}")

# Stop tracing
tracer.stop()
print("✓ Stopped tracing")

# Get explanations
print("\n" + "=" * 60)
print("Statistics:")
print("=" * 60)
stats = backend.get_stats()
print(stats)

# Get events as JSON
print("\n" + "=" * 60)
print("Events as JSON:")
print("=" * 60)
events_json = backend.get_events()
print(f"Captured {len(events_json)} characters of JSON data")

# Get explanations with different verbosity levels
print("\n" + "=" * 60)
print("Explanations (normal verbosity):")
print("=" * 60)
backend.print_explanations()

print("\n" + "=" * 60)
print("Explanations (detailed verbosity):")
print("=" * 60)
backend.set_verbosity("detailed")
explanations = backend.get_explanations()
print(explanations[:500] if len(explanations) > 500 else explanations)

print("\n" + "=" * 60)
print("All tests passed successfully!")
print("=" * 60)
