"""
WORKING EXAMPLE: Xplainit Python Bindings Demo

This demonstrates the ACTUAL working functionality as of January 12, 2026.
Run this after building: cargo build -p xplainit-python
"""

import sys
import os

# Add path to the compiled module
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'target', 'debug'))

import xplainit

print("=" * 60)
print("XPLAINIT PYTHON BINDINGS - WORKING DEMO")
print("=" * 60)
print()

# Demo 1: Basic Usage
print("📋 Demo 1: Basic API Usage")
print("-" * 60)

tracer = xplainit.Xplainit(enabled=True, verbosity="normal", output="stdout")
print(f"✅ Created tracer instance")
print(f"   Enabled: {tracer.is_enabled()}")

tracer.enable()
print(f"✅ Enabled tracing")

events = tracer.get_events()
print(f"✅ Got events (currently empty): {events}")

stats = tracer.get_stats()
print(f"✅ Got stats: {stats}")

tracer.set_verbosity("detailed")
print(f"✅ Changed verbosity to detailed")

tracer.clear()
print(f"✅ Cleared events")

tracer.disable()
print(f"✅ Disabled tracing")
print()

# Demo 2: Context Manager
print("📋 Demo 2: Context Manager")
print("-" * 60)

with xplainit.XplainitContext(enabled=True, verbosity="brief") as ctx:
    print(f"✅ Inside context manager")
    events = ctx.get_events()
    print(f"   Events: {events}")

print(f"✅ Context manager exited cleanly")
print()

# Demo 3: Module-Level Functions
print("📋 Demo 3: Module-Level Functions")
print("-" * 60)

xplainit.py_enable()
print(f"✅ Enabled globally: {xplainit.py_is_enabled()}")

xplainit.py_disable()
print(f"✅ Disabled globally: {xplainit.py_is_enabled()}")

explanation = xplainit.get_last_explanation()
print(f"✅ Last explanation: '{explanation}' (empty is expected)")
print()

# Demo 4: Different Configurations
print("📋 Demo 4: Configuration Options")
print("-" * 60)

configs = [
    ("brief", "stdout"),
    ("normal", "stderr"),
    ("detailed", "trace.log"),
    ("debug", "debug.log"),
]

for verbosity, output in configs:
    t = xplainit.Xplainit(enabled=False, verbosity=verbosity, output=output)
    print(f"✅ Created tracer: verbosity={verbosity}, output={output}")

print()

# Summary
print("=" * 60)
print("✨ ALL DEMOS COMPLETED SUCCESSFULLY!")
print("=" * 60)
print()
print("🎯 Current Status:")
print("   • Python bindings: ✅ Working")
print("   • API methods: ✅ All functional")
print("   • Context manager: ✅ Working")
print("   • Configuration: ✅ Working")
print()
print("⏳ Coming Soon (Phase 2):")
print("   • Automatic tracing via sys.settrace()")
print("   • Real-time event capture")
print("   • Decorator support (@xplainit.trace)")
print()
print("📚 For more info, see:")
print("   • PHASE1_TASK1_COMPLETE.md")
print("   • PRODUCTION_READINESS_PLAN.md")
print("="* 60)
