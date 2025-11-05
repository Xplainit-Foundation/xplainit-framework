#!/usr/bin/env python3
"""
Xplainit Python Example - Decorators

This example demonstrates using decorators for selective tracing:
- @explain decorator for specific functions
- Mixing traced and untraced code
- Performance-aware selective tracing
"""

import xplainit

@xplainit.explain_function
def fibonacci(n):
    """Calculate Fibonacci number recursively"""
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

@xplainit.explain_function
def bubble_sort(arr):
    """Sort array using bubble sort"""
    n = len(arr)
    for i in range(n):
        for j in range(0, n - i - 1):
            if arr[j] > arr[j + 1]:
                arr[j], arr[j + 1] = arr[j + 1], arr[j]
    return arr

def untraced_function():
    """This function will not be traced"""
    return "This won't show up in traces"

@xplainit.explain_function
def process_data(data):
    """Process some data with error handling"""
    if not data:
        raise ValueError("Data cannot be empty")
    
    result = []
    for item in data:
        if isinstance(item, int):
            result.append(item * 2)
        else:
            result.append(str(item).upper())
    
    return result

def main():
    print("=" * 60)
    print("Xplainit Python Example - Decorators")
    print("=" * 60)
    print()
    
    # Example 1: Fibonacci with decorator
    print("Example 1: Fibonacci (decorated function)")
    print("-" * 60)
    result = fibonacci(5)
    print(f"Fibonacci(5) = {result}")
    print()
    
    # Example 2: Untraced function (no output)
    print("Example 2: Untraced function (silent)")
    print("-" * 60)
    result = untraced_function()
    print(f"Result: {result}")
    print("(No tracing output because function not decorated)")
    print()
    
    # Example 3: Bubble sort
    print("Example 3: Bubble sort (decorated function)")
    print("-" * 60)
    arr = [64, 34, 25, 12, 22, 11, 90]
    print(f"Original: {arr}")
    sorted_arr = bubble_sort(arr.copy())
    print(f"Sorted: {sorted_arr}")
    print()
    
    # Example 4: Error in decorated function
    print("Example 4: Error in decorated function")
    print("-" * 60)
    try:
        result = process_data([])
    except ValueError as e:
        print(f"Caught error: {e}")
        explanation = xplainit.get_last_explanation()
        if explanation:
            print(f"\nXplainit explanation:\n{explanation}")
    print()
    
    # Example 5: Successful data processing
    print("Example 5: Successful data processing")
    print("-" * 60)
    data = [1, 2, "hello", 3, "world"]
    result = process_data(data)
    print(f"Input: {data}")
    print(f"Output: {result}")
    print()
    
    print("=" * 60)
    print("Decorator example complete!")
    print("=" * 60)

if __name__ == "__main__":
    main()
