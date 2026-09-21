#!/usr/bin/env python3
"""Test script to verify explanation generation works correctly."""

import sys
import os

# Add the target directory to the path
sys.path.insert(0, '/projects/sandbox/xplainit-framework/target/release')

try:
    from xplainit import Xplainit
    
    print("=" * 60)
    print("Testing Xplainit Explanation Generation")
    print("=" * 60)
    
    # Create tracer instance
    tracer = Xplainit()
    print(f"✓ Created Xplainit instance: {tracer}")
    
    # Enable tracing
    tracer.enable()
    print("✓ Enabled tracing")
    
    # Run some Python code that will be traced
    def factorial(n):
        if n <= 1:
            return 1
        return n * factorial(n - 1)
    
    result = factorial(5)
    print(f"✓ Executed factorial(5) = {result}")
    
    # Get explanations as string
    explanations = tracer.get_explanations(verbosity="detailed")
    print("\n" + "=" * 60)
    print("Generated Explanations:")
    print("=" * 60)
    print(explanations)
    
    # Print to stdout
    print("\n" + "=" * 60)
    print("Print Explanations (normal verbosity):")
    print("=" * 60)
    tracer.print_explanations()
    
    # Test with different verbosity levels
    print("\n" + "=" * 60)
    print("Minimal verbosity:")
    print("=" * 60)
    tracer.print_explanations(verbosity="minimal")
    
    # Disable tracing
    tracer.disable()
    print("\n✓ Disabled tracing")
    
    print("\n" + "=" * 60)
    print("All tests passed successfully!")
    print("=" * 60)
    
except ImportError as e:
    print(f"✗ Failed to import xplainit_python: {e}")
    print("Make sure the module is built with: cargo build --release -p xplainit-python")
    sys.exit(1)
except Exception as e:
    print(f"✗ Error during testing: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)
