#!/usr/bin/env python3
"""
Advanced Explanation Generation Demo

Demonstrates advanced features like:
- Multiple verbosity levels
- Real-time explanation printing
- Complex data structures
- Nested function calls
- Exception handling with context
- Performance metrics
"""

import sys
sys.path.insert(0, '/projects/sandbox/xplainit-framework/xplainit-python/python')

from xplainit import Xplainit
from decorators import trace, trace_class, trace_recursive, profile
import json

print("=" * 80)
print("ADVANCED EXPLANATION GENERATION DEMO")
print("=" * 80)

# Create backend with debug verbosity
backend = Xplainit(enabled=True, verbosity="debug")

# =============================================================================
# Demo 1: Complex Data Structures
# =============================================================================
print("\n" + "=" * 80)
print("Demo 1: Complex Data Structures")
print("=" * 80)

@trace(backend=backend)
def process_user_data(user_id, profile):
    """Process user data with nested structures."""
    name = profile.get('name', 'Unknown')
    age = profile.get('age', 0)
    return {'user_id': user_id, 'name': name, 'age': age}

user_profile = {
    'name': 'Alice',
    'age': 30,
    'email': 'alice@example.com',
    'preferences': {'theme': 'dark', 'language': 'en'}
}

result = process_user_data(12345, user_profile)
print(f"✓ Processed user data: {result}")

print("\nDetailed Explanation:")
backend.set_verbosity("detailed")
print(backend.get_last_explanation())

backend.clear()

# =============================================================================
# Demo 2: Nested Function Calls
# =============================================================================
print("\n" + "=" * 80)
print("Demo 2: Nested Function Calls")
print("=" * 80)

@trace(backend=backend)
def multiply(a, b):
    return a * b

@trace(backend=backend)
def calculate_area(length, width):
    return multiply(length, width)

@trace(backend=backend)
def calculate_volume(length, width, height):
    base_area = calculate_area(length, width)
    return multiply(base_area, height)

volume = calculate_volume(3, 4, 5)
print(f"✓ Calculated volume: {volume}")

print("\nExplanation of nested calls:")
backend.set_verbosity("normal")
print(backend.get_explanations())

backend.clear()

# =============================================================================
# Demo 3: Exception Handling with Context
# =============================================================================
print("\n" + "=" * 80)
print("Demo 3: Exception Handling with Context")
print("=" * 80)

@trace(backend=backend)
def validate_user(username, age):
    """Validate user with multiple checks."""
    if not username:
        raise ValueError("Username cannot be empty")
    if age < 0 or age > 150:
        raise ValueError(f"Invalid age: {age}")
    return True

# Test valid user
try:
    result = validate_user("Bob", 25)
    print(f"✓ Valid user: {result}")
except ValueError as e:
    print(f"✗ Validation error: {e}")

# Test invalid user
try:
    result = validate_user("", 25)
    print(f"✓ Valid user: {result}")
except ValueError as e:
    print(f"✗ Expected error: {e}")

# Test invalid age
try:
    result = validate_user("Charlie", 200)
    print(f"✓ Valid user: {result}")
except ValueError as e:
    print(f"✗ Expected error: {e}")

print("\nException explanations:")
backend.set_verbosity("detailed")
print(backend.get_explanations())

backend.clear()

# =============================================================================
# Demo 4: Class Methods and Inheritance
# =============================================================================
print("\n" + "=" * 80)
print("Demo 4: Class Methods and Inheritance")
print("=" * 80)

@trace_class(backend=backend)
class Animal:
    def __init__(self, name):
        self.name = name
    
    def speak(self):
        return f"{self.name} makes a sound"

@trace_class(backend=backend)
class Dog(Animal):
    def __init__(self, name, breed):
        super().__init__(name)
        self.breed = breed
    
    def speak(self):
        return f"{self.name} barks"
    
    def fetch(self, item):
        return f"{self.name} fetches {item}"

# Create and use objects
dog = Dog("Rex", "German Shepherd")
sound = dog.speak()
action = dog.fetch("ball")

print(f"✓ Dog speaks: {sound}")
print(f"✓ Dog fetches: {action}")

print("\nClass method explanations:")
backend.set_verbosity("detailed")
print(backend.get_explanations())

backend.clear()

# =============================================================================
# Demo 5: Recursive Algorithms with Depth Tracking
# =============================================================================
print("\n" + "=" * 80)
print("Demo 5: Recursive Algorithms")
print("=" * 80)

@trace_recursive(backend=backend, max_depth=10)
def fibonacci(n):
    """Calculate fibonacci number."""
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

# Calculate fibonacci(6)
result = fibonacci(6)
print(f"✓ Fibonacci(6) = {result}")

print("\nRecursive call explanations (brief):")
backend.set_verbosity("brief")
print(backend.get_explanations(verbosity="brief")[:500] + "...")

# Get statistics
events_json = backend.get_events()
events = json.loads(events_json)
print(f"\nTotal events captured: {len(events)}")

backend.clear()

# =============================================================================
# Demo 6: Performance Profiling
# =============================================================================
print("\n" + "=" * 80)
print("Demo 6: Performance Profiling")
print("=" * 80)

import time

@profile(backend=backend, name="fast_operation")
def fast_function():
    """Fast operation."""
    return sum(range(100))

@profile(backend=backend, name="slow_operation")
def slow_function():
    """Slow operation."""
    time.sleep(0.05)
    return sum(range(1000))

result1 = fast_function()
result2 = slow_function()

print(f"✓ Fast function result: {result1}")
print(f"✓ Slow function result: {result2}")

# Profile decorator outputs timing automatically

backend.clear()

# =============================================================================
# Demo 7: Different Verbosity Levels
# =============================================================================
print("\n" + "=" * 80)
print("Demo 7: Verbosity Levels Comparison")
print("=" * 80)

@trace(backend=backend)
def calculate_average(numbers):
    """Calculate average of a list."""
    if not numbers:
        return 0
    return sum(numbers) / len(numbers)

numbers = [10, 20, 30, 40, 50]
avg = calculate_average(numbers)
print(f"✓ Average: {avg}")

print("\n--- BRIEF verbosity ---")
print(backend.get_explanations(verbosity="brief"))

backend.clear()
result = calculate_average([1, 2, 3])

print("\n--- NORMAL verbosity ---")
print(backend.get_explanations(verbosity="normal"))

backend.clear()
result = calculate_average([1, 2, 3])

print("\n--- DETAILED verbosity ---")
print(backend.get_explanations(verbosity="detailed"))

backend.clear()
result = calculate_average([1, 2, 3])

print("\n--- DEBUG verbosity ---")
print(backend.get_explanations(verbosity="debug"))

backend.clear()

# =============================================================================
# Demo 8: Real-time Explanation Printing
# =============================================================================
print("\n" + "=" * 80)
print("Demo 8: Real-time Explanation Printing")
print("=" * 80)

@trace(backend=backend)
def greet(name, greeting="Hello"):
    """Greet a person."""
    return f"{greeting}, {name}!"

result = greet("World", greeting="Hi")
print(f"✓ Greeting: {result}")

print("\nReal-time explanations:")
backend.print_explanations(verbosity="normal")

backend.clear()

# =============================================================================
# Summary
# =============================================================================
print("\n" + "=" * 80)
print("ADVANCED FEATURES DEMONSTRATED")
print("=" * 80)
print()
print("✅ Complex data structures")
print("✅ Nested function calls")
print("✅ Exception handling with context")
print("✅ Class methods and inheritance")
print("✅ Recursive algorithms with depth tracking")
print("✅ Performance profiling")
print("✅ Multiple verbosity levels")
print("✅ Real-time explanation printing")
print()
print("The Xplainit framework provides comprehensive runtime code explanation!")
print("=" * 80)
