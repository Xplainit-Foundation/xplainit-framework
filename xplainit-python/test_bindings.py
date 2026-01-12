"""
Test suite for xplainit-python bindings

This tests the Rust-Python FFI layer to ensure all functions work correctly.
"""

import sys
import os

# Add the path to the compiled module
# When built with cargo, the .pyd/.so file is in target/debug
target_debug = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'target', 'debug'))
sys.path.insert(0, target_debug)
print(f"Looking for xplainit module in: {target_debug}")

try:
    import xplainit
    print("✅ Successfully imported xplainit module!")
except ImportError as e:
    print(f"❌ Failed to import xplainit: {e}")
    print("\nNote: You need to build the Python bindings first:")
    print("  cd xplainit-python")
    print("  cargo build")
    print("  # Or use maturin: maturin develop")
    sys.exit(1)

def test_basic_import():
    """Test that we can import the module"""
    assert hasattr(xplainit, 'Xplainit')
    assert hasattr(xplainit, 'XplainitContext')
    print("✅ Module has required classes")
    return True

def test_xplainit_class():
    """Test Xplainit class instantiation and methods"""
    try:
        # Create instance
        tracer = xplainit.Xplainit()
        print("✅ Created Xplainit instance")
        
        # Test enable/disable
        tracer.enable()
        assert tracer.is_enabled()
        print("✅ Enable/disable works")
        
        tracer.disable()
        assert not tracer.is_enabled()
        print("✅ State tracking works")
        
        # Test get_events
        events = tracer.get_events()
        assert isinstance(events, str)
        print(f"✅ get_events() returns: {events[:50]}...")
        
        # Test get_stats
        stats = tracer.get_stats()
        assert isinstance(stats, str)
        print(f"✅ get_stats() returns: {stats}")
        
        # Test clear
        tracer.clear()
        print("✅ clear() works")
        
        # Test set_verbosity
        tracer.set_verbosity("detailed")
        print("✅ set_verbosity() works")
        
        return True
    except Exception as e:
        print(f"❌ Error testing Xplainit class: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_context_manager():
    """Test XplainitContext as context manager"""
    try:
        ctx = xplainit.XplainitContext()
        print("✅ Created XplainitContext instance")
        
        # Test as context manager
        with ctx as c:
            events = c.get_events()
            assert isinstance(events, str)
        
        print("✅ Context manager protocol works")
        return True
    except Exception as e:
        print(f"❌ Error testing context manager: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_module_functions():
    """Test module-level functions"""
    try:
        # Test enable/disable
        xplainit.py_enable()
        print("✅ py_enable() works")
        
        assert xplainit.py_is_enabled()
        print("✅ py_is_enabled() works")
        
        xplainit.py_disable()
        print("✅ py_disable() works")
        
        # Test get_last_explanation (returns empty initially, which is correct)
        explanation = xplainit.get_last_explanation()
        assert isinstance(explanation, str)
        print(f"✅ get_last_explanation() works: '{explanation}'")
        
        return True
    except Exception as e:
        print(f"❌ Error testing module functions: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_configuration():
    """Test different configuration options"""
    try:
        # Test different verbosity levels
        tracer1 = xplainit.Xplainit(enabled=True, verbosity="brief", output="stdout")
        print("✅ Created with brief verbosity")
        
        tracer2 = xplainit.Xplainit(enabled=False, verbosity="detailed", output="stderr")
        print("✅ Created with detailed verbosity")
        
        tracer3 = xplainit.Xplainit(enabled=True, verbosity="debug", output="test.log")
        print("✅ Created with debug verbosity and file output")
        
        return True
    except Exception as e:
        print(f"❌ Error testing configuration: {e}")
        import traceback
        traceback.print_exc()
        return False

def main():
    """Run all tests"""
    print("="*60)
    print("XPLAINIT PYTHON BINDINGS TEST SUITE")
    print("="*60)
    print()
    
    tests = [
        ("Basic Import", test_basic_import),
        ("Xplainit Class", test_xplainit_class),
        ("Context Manager", test_context_manager),
        ("Module Functions", test_module_functions),
        ("Configuration", test_configuration),
    ]
    
    passed = 0
    failed = 0
    
    for name, test_func in tests:
        print(f"\n📋 Running: {name}")
        print("-" * 60)
        try:
            if test_func():
                passed += 1
            else:
                failed += 1
        except Exception as e:
            print(f"❌ Test crashed: {e}")
            failed += 1
    
    print("\n" + "="*60)
    print(f"TEST RESULTS: {passed} passed, {failed} failed")
    print("="*60)
    
    if failed == 0:
        print("\n🎉 ALL TESTS PASSED! Python bindings are working correctly!")
        return 0
    else:
        print(f"\n⚠️ {failed} test(s) failed. See details above.")
        return 1

if __name__ == "__main__":
    sys.exit(main())
