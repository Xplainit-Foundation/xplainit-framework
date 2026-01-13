"""
COMPREHENSIVE AUTOMATIC TRACING DEMO

This demonstrates the full power of Xplainit's automatic runtime tracing.
No manual instrumentation needed - just enable tracing and run your code!
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'target', 'debug'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__)))

import xplainit
from python.tracer import XplainitTracer

def demo_fibonacci():
    """Demo 1: Recursive Fibonacci with automatic tracing"""
    print("=" * 70)
    print("DEMO 1: Recursive Fibonacci (Automatic Tracing)")
    print("=" * 70)
    
    backend = xplainit.Xplainit(enabled=True, verbosity="normal")
    tracer = XplainitTracer(backend)
    
    def fibonacci(n):
        if n <= 1:
            return n
        return fibonacci(n-1) + fibonacci(n-2)
    
    print("Running: fibonacci(6)")
    print()
    
    with tracer:
        result = fibonacci(6)
    
    print(f"✅ Result: {result}")
    print(f"✅ Events captured: {backend.get_stats()}")
    print(f"✅ Last explanation: {backend.get_last_explanation()}")
    print()

def demo_sorting_algorithm():
    """Demo 2: Bubble sort algorithm tracing"""
    print("=" * 70)
    print("DEMO 2: Bubble Sort Algorithm (Automatic Tracing)")
    print("=" * 70)
    
    backend = xplainit.Xplainit(enabled=True, verbosity="brief")
    tracer = XplainitTracer(backend)
    
    def bubble_sort(arr):
        n = len(arr)
        for i in range(n):
            for j in range(0, n-i-1):
                if arr[j] > arr[j+1]:
                    arr[j], arr[j+1] = arr[j+1], arr[j]
        return arr
    
    test_array = [64, 34, 25, 12, 22, 11, 90]
    print(f"Sorting: {test_array}")
    print()
    
    with tracer:
        sorted_array = bubble_sort(test_array.copy())
    
    print(f"✅ Sorted: {sorted_array}")
    print(f"✅ Events captured: {backend.get_stats()}")
    print()

def demo_error_handling():
    """Demo 3: Error detection and explanation"""
    print("=" * 70)
    print("DEMO 3: Automatic Error Detection")
    print("=" * 70)
    
    backend = xplainit.Xplainit(enabled=True, verbosity="detailed")
    tracer = XplainitTracer(backend)
    
    def risky_division(a, b):
        return a / b
    
    def safe_calculator(x, y):
        try:
            return risky_division(x, y)
        except ZeroDivisionError:
            return float('inf')
    
    print("Testing: safe_calculator(10, 0)")
    print()
    
    with tracer:
        result = safe_calculator(10, 0)
    
    print(f"✅ Result: {result}")
    print(f"✅ Events captured: {backend.get_stats()}")
    
    # Show all events
    import json
    events = json.loads(backend.get_events())
    print(f"✅ Captured {len(events)} events including exception handling")
    print()

def demo_data_processing():
    """Demo 4: Real-world data processing pipeline"""
    print("=" * 70)
    print("DEMO 4: Data Processing Pipeline")
    print("=" * 70)
    
    backend = xplainit.Xplainit(enabled=True, verbosity="normal")
    tracer = XplainitTracer(backend)
    
    def process_user_data(users):
        """Process a list of user records"""
        def validate_email(email):
            return '@' in email and '.' in email
        
        def calculate_age(birth_year):
            return 2026 - birth_year
        
        processed = []
        for user in users:
            if validate_email(user['email']):
                user['age'] = calculate_age(user['birth_year'])
                processed.append(user)
        
        return processed
    
    test_users = [
        {'name': 'Alice', 'email': 'alice@example.com', 'birth_year': 1990},
        {'name': 'Bob', 'email': 'invalid-email', 'birth_year': 1985},
        {'name': 'Charlie', 'email': 'charlie@test.org', 'birth_year': 1995},
    ]
    
    print(f"Processing {len(test_users)} users...")
    print()
    
    with tracer:
        results = process_user_data(test_users)
    
    print(f"✅ Processed {len(results)} valid users")
    print(f"✅ Events captured: {backend.get_stats()}")
    print()

def demo_factorial_iterative():
    """Demo 5: Compare iterative vs recursive (different patterns)"""
    print("=" * 70)
    print("DEMO 5: Iterative Factorial")
    print("=" * 70)
    
    backend = xplainit.Xplainit(enabled=True, verbosity="normal")
    tracer = XplainitTracer(backend)
    
    def factorial_iterative(n):
        result = 1
        for i in range(1, n + 1):
            result *= i
        return result
    
    print("Running: factorial_iterative(10)")
    print()
    
    with tracer:
        result = factorial_iterative(10)
    
    print(f"✅ Result: {result}")
    print(f"✅ Events captured: {backend.get_stats()}")
    print(f"✅ Explanation: {backend.get_last_explanation()}")
    print()

def main():
    """Run all demos"""
    print()
    print("=" * 70)
    print("  XPLAINIT AUTOMATIC TRACING - COMPREHENSIVE DEMO".center(70))
    print("  Zero Instrumentation Runtime Code Explanation".center(70))
    print("=" * 70)
    print()
    
    demos = [
        demo_fibonacci,
        demo_sorting_algorithm,
        demo_error_handling,
        demo_data_processing,
        demo_factorial_iterative,
    ]
    
    for demo in demos:
        try:
            demo()
        except Exception as e:
            print(f"❌ Demo failed: {e}")
            import traceback
            traceback.print_exc()
        
        input("Press Enter to continue to next demo...")
        print()
    
    print("=" * 70)
    print("✨ ALL DEMOS COMPLETE!")
    print("=" * 70)
    print()
    print("🎯 What You Just Saw:")
    print("  • Automatic function call tracing")
    print("  • Argument and return value capture")
    print("  • Recursive function tracking")
    print("  • Exception detection and handling")
    print("  • Real-world code pattern analysis")
    print("  • Zero manual instrumentation needed!")
    print()
    print("🚀 This is the power of Xplainit:")
    print("   Just enable tracing and run your code normally.")
    print("   Every function call, every value, every decision is captured.")
    print("   Perfect for debugging, learning, and code understanding!")
    print()
    print("=" * 70)

if __name__ == '__main__':
    main()
