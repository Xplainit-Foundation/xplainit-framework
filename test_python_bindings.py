#!/usr/bin/env python3
"""
Test script to verify Xplainit Python bindings work correctly.
This script tests the basic functionality of the xplainit module.
"""

import sys
import os

# Add the release build directory to the Python path
sys.path.insert(0, '/projects/sandbox/xplainit-framework/target/release')

def test_import():
    """Test 1: Can we import the module?"""
    print("=" * 60)
    print("TEST 1: Importing xplainit module")
    print("=" * 60)
    
    try:
        import xplainit
        print("✓ Successfully imported xplainit module")
        return True
    except ImportError as e:
        print(f"✗ Failed to import xplainit: {e}")
        return False

def test_class_creation():
    """Test 2: Can we create an Xplainit instance?"""
    print("\n" + "=" * 60)
    print("TEST 2: Creating Xplainit instance")
    print("=" * 60)
    
    try:
        import xplainit
        tracer = xplainit.Xplainit()
        print(f"✓ Created Xplainit instance: {tracer}")
        print(f"  Type: {type(tracer)}")
        return True, tracer
    except Exception as e:
        print(f"✗ Failed to create instance: {e}")
        return False, None

def test_enable_disable(tracer):
    """Test 3: Test enable/disable functionality"""
    print("\n" + "=" * 60)
    print("TEST 3: Testing enable/disable")
    print("=" * 60)
    
    try:
        # Test initial state
        initial_state = tracer.is_enabled()
        print(f"  Initial state: {'enabled' if initial_state else 'disabled'}")
        
        # Test disable
        tracer.disable()
        disabled_state = tracer.is_enabled()
        print(f"  After disable(): {'enabled' if disabled_state else 'disabled'}")
        
        # Test enable
        tracer.enable()
        enabled_state = tracer.is_enabled()
        print(f"  After enable(): {'enabled' if enabled_state else 'disabled'}")
        
        if not disabled_state and enabled_state:
            print("✓ Enable/disable works correctly")
            return True
        else:
            print("✗ Enable/disable not working as expected")
            return False
    except Exception as e:
        print(f"✗ Error testing enable/disable: {e}")
        return False

def test_get_stats(tracer):
    """Test 4: Test get_stats() method"""
    print("\n" + "=" * 60)
    print("TEST 4: Testing get_stats()")
    print("=" * 60)
    
    try:
        stats = tracer.get_stats()
        print(f"  Stats: {stats}")
        print(f"  Type: {type(stats)}")
        
        if isinstance(stats, str) and len(stats) > 0:
            print("✓ get_stats() returns string data")
            return True
        else:
            print("✗ get_stats() returned unexpected value")
            return False
    except Exception as e:
        print(f"✗ Error testing get_stats(): {e}")
        return False

def test_get_events(tracer):
    """Test 5: Test get_events() method"""
    print("\n" + "=" * 60)
    print("TEST 5: Testing get_events()")
    print("=" * 60)
    
    try:
        events = tracer.get_events()
        print(f"  Events: {events}")
        print(f"  Type: {type(events)}")
        print(f"  Length: {len(events)}")
        
        if isinstance(events, str):
            print("✓ get_events() returns string data")
            return True
        else:
            print("✗ get_events() returned unexpected type")
            return False
    except Exception as e:
        print(f"✗ Error testing get_events(): {e}")
        return False

def test_module_functions():
    """Test 6: Test module-level functions"""
    print("\n" + "=" * 60)
    print("TEST 6: Testing module-level functions")
    print("=" * 60)
    
    try:
        import xplainit
        
        # Test module functions exist
        print("  Available module functions:")
        funcs = [attr for attr in dir(xplainit) if not attr.startswith('_')]
        for func in funcs:
            print(f"    - {func}")
        
        # Test if specific functions exist
        expected = ['Xplainit', 'py_enable', 'py_disable', 'py_is_enabled']
        found = [f for f in expected if hasattr(xplainit, f)]
        
        if len(found) == len(expected):
            print(f"✓ All expected functions found: {found}")
            return True
        else:
            print(f"✗ Missing functions: {set(expected) - set(found)}")
            return False
    except Exception as e:
        print(f"✗ Error testing module functions: {e}")
        return False

def main():
    """Run all tests"""
    print("\n" + "╔" + "=" * 58 + "╗")
    print("║" + " XPLAINIT PYTHON BINDINGS TEST SUITE ".center(58) + "║")
    print("╚" + "=" * 58 + "╝\n")
    
    results = []
    
    # Test 1: Import
    results.append(test_import())
    if not results[0]:
        print("\n✗ Cannot proceed without successful import")
        return
    
    # Test 2: Create instance
    success, tracer = test_class_creation()
    results.append(success)
    if not success:
        print("\n✗ Cannot proceed without instance")
        return
    
    # Test 3-5: Instance methods
    results.append(test_enable_disable(tracer))
    results.append(test_get_stats(tracer))
    results.append(test_get_events(tracer))
    
    # Test 6: Module functions
    results.append(test_module_functions())
    
    # Summary
    print("\n" + "╔" + "=" * 58 + "╗")
    print("║" + " TEST SUMMARY ".center(58) + "║")
    print("╠" + "=" * 58 + "╣")
    
    passed = sum(results)
    total = len(results)
    percentage = (passed / total * 100) if total > 0 else 0
    
    print("║" + f" Passed: {passed}/{total} ({percentage:.0f}%) ".center(58) + "║")
    print("╚" + "=" * 58 + "╝\n")
    
    if passed == total:
        print("✓ ALL TESTS PASSED! Python bindings are working correctly.")
    else:
        print(f"✗ {total - passed} test(s) failed. Check the output above for details.")

if __name__ == "__main__":
    main()
