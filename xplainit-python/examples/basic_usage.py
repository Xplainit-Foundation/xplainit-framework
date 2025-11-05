#!/usr/bin/env python3
"""
Xplainit Python Example - Basic Usage

This example demonstrates the basic features of Xplainit:
- Automatic function tracing
- Variable tracking
- Error explanation
- Context managers for scoped tracing
"""

import xplainit

def calculate_factorial(n):
    """Calculate factorial recursively"""
    if n <= 0:
        return 1
    return n * calculate_factorial(n - 1)

def divide_numbers(a, b):
    """Divide two numbers"""
    return a / b

def access_list(items, index):
    """Access a list element"""
    return items[index]

def main():
    print("=" * 60)
    print("Xplainit Python Example - Basic Usage")
    print("=" * 60)
    print()
    
    # Example 1: Using context manager for scoped tracing
    print("Example 1: Factorial calculation with tracing")
    print("-" * 60)
    with xplainit.XplainitContext(enabled=True, verbosity="normal"):
        result = calculate_factorial(5)
        print(f"Result: {result}")
    print()
    
    # Example 2: Global enable/disable
    print("Example 2: Division with global tracing")
    print("-" * 60)
    xplainit.enable()
    result = divide_numbers(10, 2)
    print(f"Result: {result}")
    xplainit.disable()
    print()
    
    # Example 3: Error explanation
    print("Example 3: Error explanation - Division by zero")
    print("-" * 60)
    xplainit.enable()
    try:
        result = divide_numbers(10, 0)
    except ZeroDivisionError as e:
        print(f"Caught exception: {e}")
        explanation = xplainit.get_last_explanation()
        print(f"\nXplainit explanation:\n{explanation}")
    finally:
        xplainit.disable()
    print()
    
    # Example 4: Index out of bounds
    print("Example 4: Error explanation - Index out of bounds")
    print("-" * 60)
    xplainit.enable()
    try:
        items = [1, 2, 3]
        value = access_list(items, 10)
    except IndexError as e:
        print(f"Caught exception: {e}")
        explanation = xplainit.get_last_explanation()
        print(f"\nXplainit explanation:\n{explanation}")
    finally:
        xplainit.disable()
    print()
    
    # Example 5: Using the Xplainit class
    print("Example 5: Using Xplainit class with custom settings")
    print("-" * 60)
    explainer = xplainit.Xplainit(enabled=True, verbosity="detailed", output="stdout")
    explainer.start()
    
    def greet(name):
        return f"Hello, {name}!"
    
    greeting = greet("World")
    print(f"Result: {greeting}")
    
    stats = explainer.get_stats()
    print(f"\nStatistics: {stats}")
    
    explainer.stop()
    explainer.disable()
    print()
    
    print("=" * 60)
    print("Example complete!")
    print("=" * 60)

if __name__ == "__main__":
    main()
