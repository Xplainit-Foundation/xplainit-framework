#!/usr/bin/env python3
"""
Real-World Debugging Scenario

Demonstrates how Xplainit helps debug real-world problems:
- Understanding complex algorithm execution
- Finding bugs in recursive functions
- Tracking variable values
- Identifying performance bottlenecks
"""

import sys
sys.path.insert(0, '/projects/sandbox/xplainit-framework/xplainit-python/python')

from xplainit import Xplainit
from decorators import trace, trace_recursive
import json

print("=" * 80)
print("DEBUGGING SCENARIOS WITH XPLAINIT")
print("=" * 80)

# =============================================================================
# Scenario 1: Debugging a Buggy Binary Search
# =============================================================================
print("\n" + "=" * 80)
print("Scenario 1: Debugging a Buggy Binary Search")
print("=" * 80)

backend = Xplainit(enabled=True, verbosity="detailed")

@trace_recursive(backend=backend, max_depth=20)
def binary_search(arr, target, left=0, right=None):
    """Binary search with a bug!"""
    if right is None:
        right = len(arr) - 1
    
    if left > right:
        return -1
    
    mid = (left + right) // 2
    
    if arr[mid] == target:
        return mid
    elif arr[mid] < target:
        # BUG: Should be mid + 1
        return binary_search(arr, target, mid, right)
    else:
        # BUG: Should be mid - 1
        return binary_search(arr, target, left, mid)

# Test the buggy function
arr = [1, 3, 5, 7, 9, 11, 13, 15, 17, 19]
target = 7

print(f"Searching for {target} in {arr}")
result = binary_search(arr, target)
print(f"Result: {result} (expected: 3)")

# Examine the execution trace
print("\n--- Execution Trace (showing the bug) ---")
explanations = backend.get_explanations(verbosity="normal")
print(explanations[:600] + "...")

print("\n💡 TIP: Notice the recursive calls don't adjust left/right properly!")
print("   The bug is in lines that should use 'mid + 1' and 'mid - 1'")

backend.clear()

# =============================================================================
# Scenario 2: Fixing the Bug
# =============================================================================
print("\n" + "=" * 80)
print("Scenario 2: Correct Binary Search")
print("=" * 80)

@trace_recursive(backend=backend, max_depth=20)
def binary_search_fixed(arr, target, left=0, right=None):
    """Binary search - FIXED!"""
    if right is None:
        right = len(arr) - 1
    
    if left > right:
        return -1
    
    mid = (left + right) // 2
    
    if arr[mid] == target:
        return mid
    elif arr[mid] < target:
        # FIXED: mid + 1
        return binary_search_fixed(arr, target, mid + 1, right)
    else:
        # FIXED: mid - 1
        return binary_search_fixed(arr, target, left, mid - 1)

# Test the fixed function
result = binary_search_fixed(arr, target)
print(f"✓ Result: {result} (correct!)")

print("\n--- Corrected Execution Trace ---")
explanations = backend.get_explanations(verbosity="brief")
print(explanations[:400] + "...")

backend.clear()

# =============================================================================
# Scenario 3: Understanding Algorithm Complexity
# =============================================================================
print("\n" + "=" * 80)
print("Scenario 3: Understanding Algorithm Complexity")
print("=" * 80)

@trace_recursive(backend=backend, max_depth=30)
def inefficient_fibonacci(n):
    """Naive recursive fibonacci - O(2^n) complexity."""
    if n <= 1:
        return n
    return inefficient_fibonacci(n - 1) + inefficient_fibonacci(n - 2)

# Calculate fibonacci(10)
print("Calculating fibonacci(10)...")
result = inefficient_fibonacci(10)
print(f"✓ Result: {result}")

# Get statistics
events = json.loads(backend.get_events())
print(f"\n📊 Statistics:")
print(f"   Total function calls: {len(events)}")
print(f"   This shows the exponential complexity!")

# Show call pattern
print("\n--- Call Pattern (first 10 calls) ---")
for i, event in enumerate(events[:10]):
    if 'FunctionEnter' in event:
        depth = event['FunctionEnter']['name'].split('depth=')[1].rstrip(']')
        print(f"   {i+1}. fibonacci at depth {depth}")

backend.clear()

# =============================================================================
# Scenario 4: Tracking Variable Mutations
# =============================================================================
print("\n" + "=" * 80)
print("Scenario 4: Tracking Variable Mutations")
print("=" * 80)

@trace(backend=backend)
def buggy_accumulator(numbers):
    """Accumulate numbers - with a bug!"""
    total = 0
    for i, num in enumerate(numbers):
        # BUG: Should be total += num
        total = num
    return total

numbers = [10, 20, 30, 40, 50]
result = buggy_accumulator(numbers)
print(f"Result: {result} (expected: 150)")

print("\n--- Execution Trace ---")
backend.set_verbosity("detailed")
print(backend.get_explanations())

print("\n💡 TIP: The trace shows we're not accumulating, just overwriting!")
backend.clear()

# =============================================================================
# Scenario 5: Finding the Source of an Exception
# =============================================================================
print("\n" + "=" * 80)
print("Scenario 5: Finding the Source of an Exception")
print("=" * 80)

@trace(backend=backend)
def parse_config_value(key, value):
    """Parse a configuration value."""
    if key == 'port':
        return int(value)
    elif key == 'host':
        return str(value)
    elif key == 'debug':
        return value.lower() == 'true'
    else:
        raise ValueError(f"Unknown config key: {key}")

@trace(backend=backend)
def load_config(config_dict):
    """Load configuration from dictionary."""
    result = {}
    for key, value in config_dict.items():
        result[key] = parse_config_value(key, value)
    return result

# Test with invalid config
config = {
    'host': 'localhost',
    'port': 'not_a_number',  # This will cause an error!
    'debug': 'true'
}

try:
    parsed = load_config(config)
except ValueError as e:
    print(f"✗ Error: {e}")
    
    # Get detailed error explanation
    print("\n--- Error Analysis ---")
    backend.set_verbosity("detailed")
    explanations = backend.get_explanations()
    
    # Find the exception
    lines = explanations.split('\n')
    for line in lines:
        if 'EXCEPTION' in line or 'Error:' in line:
            print(line)

backend.clear()

# =============================================================================
# Scenario 6: Performance Analysis
# =============================================================================
print("\n" + "=" * 80)
print("Scenario 6: Performance Analysis")
print("=" * 80)

import time

@trace(backend=backend)
def slow_operation():
    """A deliberately slow operation."""
    time.sleep(0.01)
    return "done"

@trace(backend=backend)
def fast_operation():
    """A fast operation."""
    return "done"

# Run multiple times
for i in range(5):
    slow_operation()
    fast_operation()

# Get execution timing
events = json.loads(backend.get_events())
print(f"✓ Total events captured: {len(events)}")

# Count slow vs fast operations
slow_count = sum(1 for e in events if 'FunctionEnter' in e and 'slow_operation' in e['FunctionEnter']['name'])
fast_count = sum(1 for e in events if 'FunctionEnter' in e and 'fast_operation' in e['FunctionEnter']['name'])

print(f"   Slow operations: {slow_count}")
print(f"   Fast operations: {fast_count}")

backend.clear()

# =============================================================================
# Summary
# =============================================================================
print("\n" + "=" * 80)
print("DEBUGGING CAPABILITIES DEMONSTRATED")
print("=" * 80)
print()
print("✅ Traced buggy binary search execution")
print("✅ Compared buggy vs fixed implementations")
print("✅ Visualized algorithm complexity")
print("✅ Tracked variable mutations")
print("✅ Identified exception sources")
print("✅ Analyzed performance patterns")
print()
print("Xplainit makes debugging easier by providing:")
print("  • Clear execution traces")
print("  • Variable value inspection")
print("  • Exception context and location")
print("  • Performance insights")
print("  • Step-by-step execution visualization")
print()
print("=" * 80)
