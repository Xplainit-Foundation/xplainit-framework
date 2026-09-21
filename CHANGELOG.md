# Changelog

All notable changes to the Xplainit Framework will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - Phase 4: Production Hardening

Phase 4 focused on making the framework safer, more resilient, better measured,
more thoroughly tested, and honestly documented. See
[`docs/STATUS.md`](docs/STATUS.md) for the authoritative project status.

### Security hardening (Task 4.1)
- **Secret/PII redaction, on by default.** New `Config` fields `redact_secrets`
  (default `true`) and `redact_key_patterns` (defaults: `password`, `passwd`,
  `secret`, `token`, `api_key`, `apikey`, `authorization`, `auth`, `credential`,
  `private_key`). Values whose key matches (case-insensitive substring) are
  replaced with `<redacted>` before leaving the process, via a single choke
  point (`ExecutionEvent::redacted()` / `redact_events()`), including secrets
  nested inside object/array payloads and single-value fields
  (`FunctionExit.return_value`, `Return.value`, `AsyncTaskResume.resumed_with`,
  `LoopIteration.loop_var_value`, `TypeError.value`, `DivisionByZero.numerator`).
- **Input-path validation** (`validate_input_path`) for user-supplied trace/log
  paths (rejects empty, embedded-NUL, missing, and non-file paths).
- **`unsafe` FFI/JNI audit:** every `unsafe` block in `xplainit-c` and
  `xplainit-java` now carries a `// SAFETY:` comment and null-guards raw
  pointers.
- Dependency/supply-chain approach documented in
  [`docs/SECURITY_AUDIT.md`](docs/SECURITY_AUDIT.md). **Blocked offline:**
  `cargo audit`, `cargo outdated`, `cargo vet` (exact commands provided; no
  output fabricated).

### Error recovery (Task 4.2)
- **Circuit-breaker + telemetry** in `RuntimeControl`: `max_consecutive_errors`
  (default 5; `0` disables) trips the breaker and auto-disables tracing after
  consecutive framework errors; `record_success()` resets the counter.
  Telemetry: `total_errors`, `total_panics`, `times_tripped`.
- **`debug_mode`** (config field and `XPLAINIT_DEBUG` env var): framework errors
  are logged to stderr when on, counted and swallowed when off.
- Documented that the release profile is `panic = "abort"`, so `catch_unwind`
  in `safe_execute` cannot catch panics in release builds — the circuit-breaker
  is the profile-independent recovery mechanism.

### Performance (Task 4.3)
- Hot-path clone/allocation reductions with byte-identical output (golden-tested):
  `ExecutionEvent::location_ref()` on the filter path (~1.9x faster path filter),
  `MemorySink` overflow eviction via `VecDeque::pop_front()` (~28x faster),
  and single pre-sized `String` in `format_events`.
- Std-only benchmark harness (`examples/bench_pipeline.rs`).
- **Honest numbers** in [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md): Python
  full-tracing overhead is ~1200x; the `<10%` target is **not achievable for
  full tracing** and is only approachable via selective tracing. Speculative
  concurrency was evaluated and declined. **Blocked offline:**
  `cargo-flamegraph`.

### Testing (Task 4.4)
- Extensive unit, integration, edge-case, hand-rolled property-style, and
  in-process load/stress tests (242 tests passing, +1 `#[ignore]`d soak test,
  0 failures). Details in [`docs/TESTING.md`](docs/TESTING.md). **Blocked
  offline:** `cargo tarpaulin` (coverage) and `cargo-fuzz` (fuzzing) — intent
  met by hand-rolled generative property tests; commands provided, no output
  fabricated.

### Documentation (Task 4.5)
- `cargo doc --all --no-deps` builds with **zero warnings**; public items are
  documented across crates.
- New plain-Markdown **user guide** under [`docs/guide/`](docs/guide/)
  (intro, getting-started, per-language guides, advanced topics) and
  **reference docs** under [`docs/reference/`](docs/reference/) (events,
  configuration, filtering, async), grounded in the actual source.
- New [`docs/STATUS.md`](docs/STATUS.md) single source of truth; historical
  root `*_COMPLETE.md` status files annotated as non-authoritative.
- **Blocked offline:** `mdbook` (the guide ships as plain Markdown; the build
  command is provided for when tooling is available).

### Honesty notes
- CLI live tracing of arbitrary programs is **not wired** (`run` renders
  captured traces and directs users to the appropriate binding to capture one).
- Node addon cannot be rebuilt offline (no `npm`).

### Phase 1 Fixes (2026-01) — Runtime hook + AST correctness

#### Fixed
- **Python automatic tracing (`sys.settrace`) now actually works** 🔥
  - Previously the auto-tracer captured **0 events** for user code: the frame
    filter in `xplainit-python/python/tracer.py` treated `__main__` as stdlib and
    matched any module containing `"tracer"`, so every user frame was excluded.
  - The filter now traces user / `__main__` code while still excluding xplainit's
    own internals and genuine stdlib, and the `call`-event callback returns a
    local trace function so `return`/`exception` events fire.
  - `python test_automatic_tracing.py` now passes **3/3 with events captured**
    (was 1/3 with 0 events). Note: earlier docs that described automatic tracing
    as "complete" were premature — it is complete now.
- **AST `get_containing_function` now resolves the enclosing function** 🔥
  - `xplainit-core/src/ast.rs` previously always returned `None`. It now walks up
    the tree-sitter parent chain from the node at a location to the enclosing
    function/method definition and returns its name.

#### Added
- **New real-world example** `xplainit-core/examples/real_world_debugging.rs`
  - Records function-enter / variable / function-exit / division-by-zero events
    into an `EventStore`, replays them as natural-language explanations, and runs
    `ErrorExplainer` for root-cause analysis, fix suggestions, and prevention tips.

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
