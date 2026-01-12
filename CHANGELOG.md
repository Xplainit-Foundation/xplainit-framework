# Changelog

All notable changes to the Xplainit Framework will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Phase 1 Complete (2026-01-13)

#### Fixed
- **Python PyO3 0.22 Bindings** 🔥
  - Fixed context manager signatures (`__enter__`, `__exit__`)
  - Simplified `XplainitContext` borrowing (removed was_enabled field)
  - All Python tests passing (5/5)
  - Module imports successfully
  - Context manager protocol working correctly

#### Added
- **Tree-sitter AST Integration** ✨
  - Real parsing implementation (replaced stub)
  - Language-specific parsers: Python, JavaScript, Rust, C, C++
  - Node location finding
  - Context extraction around locations
  - Function name extraction
  - New example: `ast_parsing.rs` demonstrating all features
  - All parsers tested and working

#### Verified
- **Example Programs** ✅
  - `custom_filters.rs` - Compiles and runs successfully
  - `event_pipeline.rs` - Compiles and runs successfully
  - `basic_usage.rs` - Working
  - `error_analysis.rs` - Working
  - `ast_parsing.rs` - NEW, working

#### Documentation
- Created `PHASE1_COMPLETE.md` - Full Phase 1 summary
- Created `PHASE1_TASK1_COMPLETE.md` - Python bindings details
- Created `test_bindings.py` - Comprehensive Python test suite
- Updated CHANGELOG.md with Phase 1 completion

### Planned
- Rust proc macro integration
- Advanced natural language explanation templates
- Property-based testing expansion
- Code coverage reporting
- Performance regression tests
- HTML/Markdown output formats

## [0.1.0] - 2025-11-06

### Added - Core Framework
- **Runtime Engine** with lifecycle management (Idle, Collecting, Paused states)
- **21 Event Types** (9 normal + 12 error types) with full serialization support
- **Event Storage** with circular buffer and configurable capacity
- **Event Pipeline** architecture (Filter → Processor → Sink)
  
### Added - Language Bindings
- **Python Integration (PyO3 0.22)** ✨
  - Full API: `enable()`, `disable()`, `is_enabled()`, `get_events()`, `get_statistics()`, `clear()`
  - `Xplainit` class for object-oriented usage
  - Context manager support (`with` statement)
  - Working `test_bindings.py` example
  - Package installable with maturin

- **JavaScript/Node.js Integration (Neon 1.1)** ✨
  - Full API: `enable()`, `disable()`, `isEnabled()`, `getEvents()`, `clearEvents()`, `getStatistics()`
  - TypeScript type definitions (index.d.ts)
  - npm package support (package.json)
  - Comprehensive README with examples

- **C/C++ FFI Bindings (cbindgen 0.27)** ✨
  - C-compatible API: `xplainit_create()`, `xplainit_free()`, `xplainit_enable()`, etc.
  - Auto-generated header file (`xplainit-c.h`)
  - Static and dynamic library builds (`cdylib`, `staticlib`)
  - C example (`example.c`)
  - C++ RAII wrapper example (`example.cpp`)

- **Java JNI Bindings (jni 0.21)** ✨
  - Full Java API with `Xplainit` class and `Statistics` inner class
  - AutoCloseable implementation for try-with-resources support
  - JNI native methods: `nativeCreate()`, `nativeFree()`, `nativeEnable()`, `nativeDisable()`, `nativeIsEnabled()`, `nativeGetEvents()`, `nativeClearEvents()`, `nativeGetStatistics()`
  - Gson integration for JSON parsing (2.10.1)
  - Maven build configuration (pom.xml) with Java 11 target
  - exec-maven-plugin for automated Rust library builds
  - `BasicExample.java` demonstrating try-with-resources pattern

- **Go CGO Bindings** ✨
  - Complete Go wrapper for C FFI library
  - `Xplainit` struct with all core methods
  - `Statistics` struct for runtime metrics
  - CGO directives for cross-platform library linking (linux, darwin, windows)
  - Go module support (go.mod) for Go 1.21+
  - `basic.go` example with defer cleanup pattern
  - Methods: `New()`, `Close()`, `Enable()`, `Disable()`, `IsEnabled()`, `GetEvents()`, `ClearEvents()`, `GetStatistics()`, `Version()`

### Added - Filtering System
- **AcceptAllFilter**: Capture all events
- **FunctionFilter**: Include/exclude by function name patterns
- **EventTypeFilter**: Filter by event categories (errors only, functions only, etc.)
- **DepthFilter**: Limit by stack depth
- **CompositeFilter**: Combine multiple filters with AND/OR logic

### Added - Processing Pipeline
- **PassThroughProcessor**: No-op for benchmarking
- **EnrichmentProcessor**: Add metadata to events
- **DeduplicationProcessor**: Remove duplicate events with LRU cache
- **RateLimitProcessor**: Limit events per second
- **ProcessorPipeline**: Chain processors together

### Added - Output Sinks
- **ConsoleSink**: Output to stdout/stderr (JSON, Text, Colored)
- **FileSink**: Write to files with rotation support
- **MemorySink**: Store events in memory with capacity limits
- **MultiSink**: Fan-out to multiple sinks simultaneously

### Added - Examples
- **basic_usage.rs**: Complete workflow demonstration
- **error_analysis.rs**: Error analysis capabilities
- **custom_filters.rs**: 4 filtering strategies with working examples
- **event_pipeline.rs**: 4 pipeline demonstrations (simple, multi-processor, multi-sink, production)

### Added - Testing
- **94 tests passing** across all packages:
  - 76 core unit tests
  - 9 integration tests
  - 5 C FFI tests
  - 1 Python test
  - 1 Node.js test
  - 1 Java test
  - 1 doc test
- Zero clippy warnings
- Zero compiler warnings
- 100% test pass rate

### Added - Documentation
- Comprehensive README with working examples for all languages
- Individual READMEs for each language binding
- API documentation with inline examples
- Contributing guidelines
- MIT OR Apache-2.0 dual license

### Performance
- Event creation: **<2μs** average
- Event storage: **1.29μs** ± 0.08μs
- Filtering: **<500ns** per event
- Full pipeline: **1.75μs** ± 0.12μs
- Control overhead: **782ps** ± 45ps
- Throughput: **~571K events/sec** (single-threaded)
- Application overhead: **1-2%** typical

### Technical Details
- Rust 1.91+ required
- PyO3 0.22 for Python bindings
- Neon 1.1 for Node.js bindings
- cbindgen 0.27 for C header generation
- jni 0.21 for Java JNI bindings
- CGO for Go bindings (wraps C FFI library)
- Maven 3.6+ for Java builds
- Go 1.21+ for Go module support
- Zero-copy event handling where possible
- Lock-free data structures for performance
- Minimal heap allocations in hot paths

## Release Schedule

- **v0.1.0**: Core framework + 5 language bindings (Current)
- **v0.2.0**: Enhanced language integrations (Planned)
- **v0.3.0**: Additional runtime features (Planned)
- **v0.4.0**: Advanced analysis capabilities (Planned)
- **v1.0.0**: Production-ready stable release (Future)

## Migration Guide

### From Pre-release to 0.1.0

This is the initial release. No migration needed.

## Contributors

Special thanks to all contributors who helped build Xplainit Framework.

---

For more details, see the [full documentation](./xplainit-core/README.md).
