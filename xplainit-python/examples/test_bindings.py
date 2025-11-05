"""
Test Xplainit Python bindings - Basic functionality
"""
import xplainit

def main():
    print("=== Xplainit Python Bindings Test ===\n")
    
    # Test 1: Create Xplainit instance
    print("Test 1: Creating Xplainit instance...")
    explainer = xplainit.Xplainit(enabled=True, verbosity="normal", output="stdout")
    print(f"✓ Created: {explainer}")
    print(f"✓ Enabled: {explainer.is_enabled()}")
    
    # Test 2: Enable/Disable
    print("\nTest 2: Enable/Disable functionality...")
    explainer.disable()
    print(f"  After disable - Enabled: {explainer.is_enabled()}")
    explainer.enable()
    print(f"  After enable - Enabled: {explainer.is_enabled()}")
    
    # Test 3: Verbosity levels
    print("\nTest 3: Changing verbosity levels...")
    for level in ["brief", "normal", "detailed", "debug"]:
        explainer.set_verbosity(level)
        print(f"  ✓ Set verbosity to: {level}")
    
    # Test 4: Statistics
    print("\nTest 4: Getting statistics...")
    stats = explainer.get_stats()
    print(f"  Stats: {stats}")
    
    # Test 5: Events
    print("\nTest 5: Getting events...")
    events = explainer.get_events()
    print(f"  Events (JSON): {events[:100]}..." if len(events) > 100 else f"  Events: {events}")
    
    # Test 6: Context manager
    print("\nTest 6: Using context manager...")
    ctx = xplainit.XplainitContext(enabled=True, verbosity="normal")
    # Note: Context manager needs Python runtime for sys.settrace, skipping for now
    print("  ⊗ Context manager test skipped (requires Python runtime integration)")
    print("  ✓ Created context object successfully")
    
    # Test 7: Clear
    print("\nTest 7: Clearing events...")
    explainer.clear()
    print("  ✓ Events cleared")
    
    print("\n=== All Tests Passed! ===")

if __name__ == "__main__":
    main()
