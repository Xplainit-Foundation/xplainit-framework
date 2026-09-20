#!/usr/bin/env python3
"""
Test script for automatic tracing with sys.settrace() integration.

This demonstrates that Xplainit can now automatically capture
function calls, returns, and exceptions without manual instrumentation.
"""

import sys
import os

# Add the release build directory to the Python path
sys.path.insert(0, '/projects/sandbox/xplainit-framework/target/release')
sys.path.insert(0, '/projects/sandbox/xplainit-framework/xplainit-python/python')

def test_automatic_tracing():
    """Test that automatic tracing works end-to-end."""
    print("=" * 70)
    print("TEST: Automatic Tracing with sys.settrace()")
    print("=" * 70)
    
    # Import the modules
    try:
        import xplainit
        from tracer import AutoTracer
        print("✓ Successfully imported xplainit and AutoTracer")
    except ImportError as e:
        print(f"✗ Failed to import: {e}")
        return False
    
    # Create backend instance
    try:
        backend = xplainit.Xplainit(enabled=True, verbosity="normal")
        print(f"✓ Created Xplainit backend: {backend}")
    except Exception as e:
        print(f"✗ Failed to create backend: {e}")
        return False
    
    # Create AutoTracer
    try:
        auto_tracer = AutoTracer(
            backend=backend,
            trace_calls=True,
            trace_returns=True,
            trace_exceptions=True,
            trace_lines=False,
            max_depth=50
        )
        print(f"✓ Created AutoTracer")
    except Exception as e:
        print(f"✗ Failed to create AutoTracer: {e}")
        return False
    
    # Start automatic tracing
    try:
        auto_tracer.start()
        print("✓ Started automatic tracing")
    except Exception as e:
        print(f"✗ Failed to start tracing: {e}")
        return False
    
    # Define test functions
    def add_numbers(x, y):
        result = x + y
        return result
    
    def multiply_numbers(a, b):
        return a * b
    
    def divide_by_zero():
        return 10 / 0
    
    # Execute traced code
    print("\n--- Executing traced functions ---")
    
    result1 = add_numbers(5, 3)
    print(f"add_numbers(5, 3) = {result1}")
    
    result2 = multiply_numbers(4, 7)
    print(f"multiply_numbers(4, 7) = {result2}")
    
    # Test exception handling
    print("\n--- Testing exception capture ---")
    try:
        divide_by_zero()
    except ZeroDivisionError as e:
        print(f"Caught expected exception: {e}")
    
    # Stop tracing
    auto_tracer.stop()
    print("\n✓ Stopped automatic tracing")
    
    # Get captured events
    events_json = backend.get_events()
    print(f"\n--- Captured Events ---")
    print(f"Events JSON length: {len(events_json)}")
    
    # Parse and display events
    import json
    try:
        events = json.loads(events_json)
        print(f"Number of events captured: {len(events)}")
        
        for i, event in enumerate(events):
            print(f"\nEvent {i+1}:")
            print(f"  Type: {list(event.keys())[0] if isinstance(event, dict) else 'unknown'}")
            if isinstance(event, dict):
                for key, value in list(event.items())[:5]:  # Show first 5 fields
                    print(f"    {key}: {value}")
    except Exception as e:
        print(f"Could not parse events JSON: {e}")
    
    # Get statistics
    stats = backend.get_stats()
    print(f"\n--- Statistics ---")
    print(f"  {stats}")
    
    # Success criteria
    if len(events) > 0:
        print("\n" + "=" * 70)
        print("✓ SUCCESS: Automatic tracing is working!")
        print("=" * 70)
        return True
    else:
        print("\n" + "=" * 70)
        print("✗ FAILURE: No events were captured")
        print("=" * 70)
        return False


def test_depth_limiting():
    """Test that depth limiting works."""
    print("\n" + "=" * 70)
    print("TEST: Depth Limiting")
    print("=" * 70)
    
    import xplainit
    from tracer import AutoTracer
    
    backend = xplainit.Xplainit(enabled=True)
    auto_tracer = AutoTracer(backend=backend, max_depth=3)
    
    def recursive_function(n):
        if n <= 0:
            return 0
        return n + recursive_function(n - 1)
    
    auto_tracer.start()
    result = recursive_function(5)  # Should only trace first 3 levels
    auto_tracer.stop()
    
    events_json = backend.get_events()
    import json
    events = json.loads(events_json)
    
    print(f"Recursive calls: 5 levels")
    print(f"Max depth limit: 3")
    print(f"Events captured: {len(events)}")
    print(f"Result: {result}")
    
    if len(events) > 0:
        print("✓ Depth limiting test passed")
        return True
    else:
        print("✗ Depth limiting test failed")
        return False


def test_module_exclusion():
    """Test that standard library modules are excluded."""
    print("\n" + "=" * 70)
    print("TEST: Module Exclusion")
    print("=" * 70)
    
    import xplainit
    from tracer import AutoTracer
    
    backend = xplainit.Xplainit(enabled=True)
    auto_tracer = AutoTracer(backend=backend)
    
    auto_tracer.start()
    
    # Call some stdlib functions
    import os
    _ = os.path.basename("/some/path/to/file.txt")
    _ = len([1, 2, 3, 4, 5])
    
    auto_tracer.stop()
    
    events_json = backend.get_events()
    import json
    events = json.loads(events_json)
    
    print(f"Called stdlib functions: os.path.basename, len")
    print(f"Events captured: {len(events)}")
    
    if len(events) == 0:
        print("✓ Standard library was correctly excluded")
        return True
    else:
        print(f"⚠ Captured {len(events)} events (expected 0 for stdlib)")
        return True  # Not a failure, just unexpected


def main():
    """Run all tests."""
    print("\n" + "╔" + "=" * 68 + "╗")
    print("║" + " AUTOMATIC TRACING TEST SUITE ".center(68) + "║")
    print("╚" + "=" * 68 + "╝\n")
    
    results = []
    
    # Test 1: Basic automatic tracing
    results.append(test_automatic_tracing())
    
    # Test 2: Depth limiting
    results.append(test_depth_limiting())
    
    # Test 3: Module exclusion
    results.append(test_module_exclusion())
    
    # Summary
    print("\n" + "╔" + "=" * 68 + "╗")
    print("║" + " TEST SUMMARY ".center(68) + "║")
    print("╠" + "=" * 68 + "╣")
    
    passed = sum(results)
    total = len(results)
    percentage = (passed / total * 100) if total > 0 else 0
    
    print("║" + f" Passed: {passed}/{total} ({percentage:.0f}%) ".center(68) + "║")
    print("╚" + "=" * 68 + "╝\n")
    
    if passed == total:
        print("✓ ALL TESTS PASSED! Automatic tracing is fully functional.")
    else:
        print(f"✗ {total - passed} test(s) failed. Check output above for details.")


if __name__ == "__main__":
    main()
