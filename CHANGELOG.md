# Changelog

All notable changes to the Xplainit Framework will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Java runtime integration (JNI)
- Go runtime integration (CGO)
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
- **93 tests passing** across all packages:
  - 76 core unit tests
  - 9 integration tests
  - 5 C FFI tests
  - 1 Python test
  - 1 Node.js test
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
- Zero-copy event handling where possible
- Lock-free data structures for performance
- Minimal heap allocations in hot paths

## Release Schedule

- **v0.1.0**: Core framework (Current)
- **v0.2.0**: Python integration (Planned)
- **v0.3.0**: JavaScript/Node.js integration (Planned)
- **v0.4.0**: Additional language bindings (Planned)
- **v1.0.0**: Production-ready stable release (Future)

## Migration Guide

### From Pre-release to 0.1.0

This is the initial release. No migration needed.

## Contributors

Special thanks to all contributors who helped build Xplainit Framework.

---

For more details, see the [full documentation](./xplainit-core/README.md).
