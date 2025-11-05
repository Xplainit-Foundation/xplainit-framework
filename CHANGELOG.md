# Changelog

All notable changes to the Xplainit Framework will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Python runtime integration (PyO3 bindings)
- JavaScript/Node.js runtime integration (N-API)
- Java runtime integration (JNI)
- C/C++ runtime integration
- Go runtime integration
- Property-based testing
- Code coverage reporting
- Performance regression tests

## [0.1.0] - 2025-11-05

### Added
- **Core Framework**
  - Runtime engine with lifecycle management (Idle, Collecting, Paused states)
  - Event capture system supporting 21 event types (9 normal + 12 error types)
  - Event storage with circular buffer and configurable capacity
  - Event pipeline architecture (Filter → Processor → Sink)
  
- **Natural Language Generation**
  - ExplanationGenerator with 3 verbosity levels (Brief, Normal, Detailed)
  - Context-aware explanations with variable values and location information
  - Support for all event types with customized output

- **Error Analysis**
  - ErrorExplainer with root cause analysis
  - Automatic fix suggestions based on error patterns
  - Prevention tips and best practices
  - Learning resources for common errors
  - Analysis of leading events contributing to errors

- **Advanced Filtering**
  - FunctionFilter (include/exclude patterns)
  - ModuleFilter (path-based filtering)
  - EventTypeFilter (by event category)
  - RegexFilter (pattern matching)
  - CallStackFilter (depth limiting)
  - PerformanceFilter (time-based filtering)
  - AdvancedFilter (combining multiple filters)

- **Output Formatting**
  - TextFormatter with color support
  - JsonFormatter with pretty printing
  - HtmlFormatter with syntax highlighting
  - StructuredFormatter with custom templates

- **Pipeline Components**
  - Processors: PassThrough, Enrichment, Deduplication, RateLimiting
  - Sinks: Memory, Console, File (with rotation)
  - Flexible composition and chaining

- **Control System**
  - Runtime enable/disable
  - Performance impact control (<2% overhead)
  - Event collection pause/resume
  - Dynamic configuration updates

- **Performance Optimizations**
  - Event creation: 1.97μs average
  - Event storage: 1.29μs average
  - Filtering: <500ns per event
  - Full pipeline: 1.75μs average
  - Control check: 782ps overhead
  - Application overhead: 1-2%

- **Testing**
  - 76 unit tests covering all modules
  - 9 integration tests for multi-component workflows
  - Zero clippy warnings
  - Zero compiler warnings
  - 100% test pass rate

- **Documentation**
  - Comprehensive README with architecture diagrams
  - 4 working examples:
    - basic_usage.rs: Complete workflow demonstration
    - error_analysis.rs: Error analysis capabilities
    - custom_filters.rs: Advanced filtering patterns
    - event_pipeline.rs: Pipeline construction
  - API documentation for all modules
  - Contributing guidelines
  - MIT License

### Performance
- Event creation: 1.97μs ± 0.15μs
- Event storage: 1.29μs ± 0.08μs
- Filtering: <500ns per event
- Full pipeline: 1.75μs ± 0.12μs
- Control overhead: 782ps ± 45ps
- Throughput: ~571K events/sec (single-threaded)
- Application overhead: 1-2% typical

### Technical Details
- Rust 1.91+ required
- Zero-copy event handling where possible
- Lock-free data structures for performance
- Minimal heap allocations in hot paths
- Async-ready architecture (not yet enabled)

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
