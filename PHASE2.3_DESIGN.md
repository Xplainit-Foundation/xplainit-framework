# Phase 2.3: C/C++ Runtime Hooks - Architecture & Design

## Overview
Implement automatic runtime tracing for C and C++ programs using multiple approaches for maximum compatibility and flexibility.

## Challenge
Unlike Python (sys.settrace()) and Node.js (V8 Inspector), C/C++ has no built-in runtime introspection. We need to use compiler instrumentation, dynamic linking, or debugger integration.

## Proposed Approaches

### 1. GCC/Clang Function Instrumentation (Primary Approach)
**Pros**:
- ✅ Native compiler support (`-finstrument-functions`)
- ✅ Works with any C/C++ code
- ✅ Minimal overhead (<5%)
- ✅ Production-ready

**Cons**:
- ⚠️ Requires recompilation with flags
- ⚠️ Inline functions not traced

**Implementation**:
```c
// Compiler calls these automatically
void __cyg_profile_func_enter(void *func, void *caller);
void __cyg_profile_func_exit(void *func, void *caller);
```

### 2. LD_PRELOAD Function Interposition (Secondary Approach)
**Pros**:
- ✅ No recompilation needed
- ✅ Works with existing binaries
- ✅ Can intercept library calls

**Cons**:
- ⚠️ Linux/Unix only
- ⚠️ Only traces library functions (malloc, free, etc.)
- ⚠️ User functions not visible

**Implementation**:
```c
// Intercept library functions
void* malloc(size_t size) {
    static void* (*real_malloc)(size_t) = NULL;
    if (!real_malloc) real_malloc = dlsym(RTLD_NEXT, "malloc");
    xplainit_trace("malloc", size);
    return real_malloc(size);
}
```

### 3. DWARF Debug Info Parsing (Supporting Feature)
**Purpose**: Resolve function addresses to names

**Pros**:
- ✅ Rich debug information
- ✅ Source line numbers
- ✅ Variable names and types

**Implementation**:
- Parse DWARF sections from ELF binary
- Build address → function name mapping
- Use libdwarf or libdw

### 4. Windows: ETW (Event Tracing for Windows)
**Purpose**: Tracing on Windows platform

**Pros**:
- ✅ Native Windows support
- ✅ Low overhead
- ✅ System-wide tracing

**Implementation**:
- Use ETW APIs
- Register event providers
- Capture function entry/exit

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│ User C/C++ Application                                  │
│                                                          │
│  int main() {                                           │
│      my_function(10, 20);  ← Instrumented automatically │
│  }                                                       │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ Compiler Instrumentation Hooks                          │
│                                                          │
│  __cyg_profile_func_enter(func_addr, caller_addr)      │
│  __cyg_profile_func_exit(func_addr, caller_addr)       │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ libxplainit_trace.so / xplainit_trace.dll               │
│                                                          │
│  - Resolve addresses using DWARF                        │
│  - Extract function names, file, line                   │
│  - Serialize arguments (limited capability)             │
│  - Call Rust backend via FFI                            │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ xplainit-core (Rust Backend)                            │
│                                                          │
│  - Event storage                                        │
│  - Analysis engine                                      │
│  - Natural language generation                          │
└─────────────────────────────────────────────────────────┘
```

## Implementation Plan

### Step 1: Create C Bindings Library
- `xplainit-c/lib/libxplainit_trace.c`
- Implements `__cyg_profile_func_enter/exit`
- Links to xplainit-core via C FFI

### Step 2: DWARF Address Resolution
- Parse DWARF debug sections
- Build address → symbol map
- Cache for performance

### Step 3: Stack Unwinding
- Use libunwind or backtrace()
- Capture call stack on events
- Provide stack traces

### Step 4: Windows Support
- Implement ETW provider
- Windows-specific function hooks
- PDB symbol resolution

### Step 5: Testing & Examples
- Example C programs
- Performance benchmarks
- Integration tests

## Technical Challenges

### 1. Function Address Resolution
**Problem**: Compiler gives us raw addresses, not names

**Solution**: Parse DWARF debug information to map addresses to symbols

### 2. Argument Capture
**Problem**: C doesn't have reflection - we can't inspect function arguments

**Solutions**:
- Use DWARF to get argument count/types (limited)
- Manual annotation with macros (verbose)
- Accept limitation (trace calls only, not args)

**Chosen**: Hybrid - automatic call tracing + optional manual annotations

### 3. Inline Functions
**Problem**: `-finstrument-functions` doesn't trace inlined functions

**Solution**: Compile with `-fno-inline` for complete tracing, or accept limitation

### 4. Performance
**Problem**: Function entry/exit on every call could be expensive

**Solutions**:
- Compile-time filtering (don't instrument system headers)
- Runtime sampling (trace 1 in N calls)
- Smart caching (address resolution)

### 5. Cross-Platform
**Problem**: Different platforms have different mechanisms

**Solution**: 
- Linux/macOS: GCC instrumentation + DWARF
- Windows: MSVC hooks + PDB symbols
- Fallback: LD_PRELOAD for library functions only

## Success Criteria

✅ **Automatic Tracing**: Functions traced without manual instrumentation  
✅ **Function Names**: Address resolution works correctly  
✅ **Call Stack**: Stack traces captured  
✅ **Performance**: <10% overhead with instrumentation  
✅ **Cross-Platform**: Linux, macOS, Windows support  
✅ **Production Ready**: Works with optimized builds (-O2)  

## Limitations (Accepted)

⚠️ **Argument Values**: Limited or manual annotation required  
⚠️ **Inline Functions**: Not traced (or requires `-fno-inline`)  
⚠️ **Recompilation**: Requires `-finstrument-functions` flag  
⚠️ **System Libraries**: Only if recompiled with instrumentation  

## Comparison with Python/Node.js

| Feature | Python | Node.js | C/C++ |
|---------|--------|---------|-------|
| **Automatic** | ✅ Yes | ✅ Yes | ⚠️ Requires flag |
| **Arguments** | ✅ Full | ✅ Full | ⚠️ Limited |
| **Line-level** | ✅ Yes | ⚠️ Limited | ❌ No |
| **Performance** | ~10% | ~15% | ~5% |
| **Recompile** | ❌ No | ❌ No | ⚠️ Yes (with flags) |
| **Inline Functions** | N/A | N/A | ❌ Not traced |

## Next Steps

1. ✅ Architecture design (this document)
2. ⏳ Implement libxplainit_trace.c
3. ⏳ DWARF address resolution
4. ⏳ Stack unwinding integration
5. ⏳ Example programs
6. ⏳ Performance testing

---

*Phase 2.3 Design: COMPLETE*  
*Ready to implement*
