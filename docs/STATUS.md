# Project Status (Single Source of Truth)

This file is the **authoritative** status for the Xplainit framework. The many
`*_COMPLETE.md` / `*_SUCCESS.md` / summary files at the repository root are
historical and have, in places, **overstated** completeness. Where they conflict
with this document, **this document wins**.

_Last updated: Phase 4 (Production Hardening)._

## Build and test gate (verified in the offline sandbox)

The following all pass on the current branch:

- `cargo build --release`
- `cargo test` (242 tests passing, +1 `#[ignore]`d soak test, 0 failures)
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo fmt --all -- --check`
- `cargo doc --all --no-deps` (zero warnings)
- Python suites: `test_automatic_tracing.py` (3/3), `test_frame_filter.py`
  (11/11), `test_depth_accounting.py` (23/23), `test_decorators.py` (pass),
  after `maturin develop --release` in `xplainit-python`.

## What is actually done

### Core (`xplainit-core`)
- Event model (`ExecutionEvent`, `Value`, `SourceLocation`, `StackFrame`).
- Filtering: two layers (`EventFilter` implementations + `AdvancedFilter`).
- Pipeline: filter → processor → sink; bounded `EventStore` with stats.
- Formatters: text, JSON, HTML, Markdown.
- Explanations and error analysis (`ExplanationGenerator`, `ErrorExplainer`).
- Async task reconstruction (`AsyncTaskTracker`).
- **Phase 4 security:** secret redaction on by default (`redact_secrets`,
  `redact_key_patterns`), applied at a single choke point including nested
  payloads; input-path validation.
- **Phase 4 error recovery:** circuit-breaker + telemetry in `RuntimeControl`
  (`max_consecutive_errors`, `total_errors`, `total_panics`, `times_tripped`,
  `debug_mode`). The breaker is driven end-to-end by `EventPipeline`: attach a
  control with `EventPipeline::with_control(control)` and every `handle_event`
  routes processor/sink failures through `record_error` (tripping the breaker
  after N consecutive errors) and clean events through `record_success`. See
  the "circuit-breaker integration point" note under limitations for how this
  is wired.
- **Phase 4 performance:** hot-path clone/allocation reductions (see
  [PERFORMANCE.md](PERFORMANCE.md)).
- **Phase 4 testing:** extensive unit/integration/property-style/load tests
  (see [TESTING.md](TESTING.md)).

### Python (`xplainit-python`)
- Native module (`import xplainit`) via PyO3/maturin.
- Automatic tracing via `sys.settrace()` — genuinely captures user-code events
  (the earlier "0 events" frame-filter bug was fixed in Phase 1).
- Decorators (`@trace`, `@trace_class`, `@profile`, `@trace_recursive`).
- Verified by the four Python suites listed above.

### CLI (`xplainit-cli`)
- `analyze <log.json>` — error analysis over a JSON event log.
- `report <trace.json> --format text|json|html|markdown` — render a trace.
- `explain <file::function> --trace <trace.json>` — explain recorded events.
- `run <trace.json>` — renders a captured trace.

### Dashboard (`xplainit-dashboard`)
- Loads a trace JSON and serves fixed embedded routes (`/`, `/style.css`,
  `/app.js`, `/summary`, `/callgraph`, `/timeline`, `/tasks`, `/events`).
- No dynamic file serving; request-head size cap (64KB), 15s read timeout,
  rejects non-GET.

### Other bindings
- **C/C++** (`xplainit-c`): C FFI (`cdylib`/`staticlib`), cbindgen header, all
  `unsafe` blocks documented with `// SAFETY:` and null-guarded.
- **Java** (`xplainit-java`): JNI bindings, Maven build, `unsafe` documented.
- **Go**: CGO wrapper around the C FFI.
- **Node** (`xplainit-node`): Neon addon; **cannot be rebuilt offline** (no npm).

## Honest limitations

- **CLI live tracing is not wired.** `xplainit run some_program.py` does **not**
  attach a live tracer; it identifies the language and directs you to the
  binding to capture a trace. The `attach` style of live `explain` is likewise
  not wired; `explain --trace <trace.json>` works on a captured trace.
- **Python full-tracing overhead is ~1200x, not `<10%`.** The `<10%` target is
  **not achievable for full call-granularity tracing** because a global
  `sys.settrace()` callback fires on every call/return. Low overhead requires
  selective tracing (decorators, narrow modules, shallow depth). See
  [PERFORMANCE.md](PERFORMANCE.md).
- **No `AsyncTaskComplete` event.** Task completion is not derivable from events
  alone; tasks end `Running`/`Awaiting` unless `mark_completed` is called
  out-of-band. See [reference/async.md](reference/async.md).
- Non-Python bindings expose control/recording APIs, not full automatic source
  tracers for arbitrary programs.
- **Circuit-breaker integration point.** The breaker's driver lives in
  `EventPipeline`: a `RuntimeControl` attached with
  `EventPipeline::with_control(control)` makes `handle_event` record every
  processor/sink success and failure, so consecutive real framework errors trip
  the breaker and auto-disable tracing (covered end-to-end by
  `pipeline::tests::test_pipeline_control_trips_breaker_after_consecutive_errors`
  and `..._success_resets_consecutive_counter`). The framework does **not**
  construct a global pipeline for you and therefore does not auto-attach a
  control; each embedder (a binding, the CLI capture path, or an application)
  builds its `EventPipeline` and opts in via `with_control`. Until an embedder
  does so, `record_error`/`record_success` are not invoked for that pipeline
  and the breaker stays idle for it. This is a deliberate wiring point, not
  active-by-default behavior.

## Blocked by the offline (`INTEGRATIONS_ONLY`) sandbox

These tools cannot be installed or run here; the docs give the exact commands to
run when online and **do not fabricate their output**:

| Tool | Purpose | Where documented |
| --- | --- | --- |
| `cargo audit` | RustSec advisory scan | [SECURITY_AUDIT.md](SECURITY_AUDIT.md) |
| `cargo outdated` / `cargo vet` | dependency freshness / supply chain | [SECURITY_AUDIT.md](SECURITY_AUDIT.md) |
| `cargo tarpaulin` | line/branch coverage | [TESTING.md](TESTING.md) |
| `cargo-fuzz` | coverage-guided fuzzing (needs nightly) | [TESTING.md](TESTING.md) |
| `cargo-flamegraph` | profiling | [PERFORMANCE.md](PERFORMANCE.md) |
| `mdbook` | render the user guide as HTML | [guide/README.md](guide/README.md) |
| `npm` | rebuild the Node addon | above |

## Where to read more

- User guide: [guide/README.md](guide/README.md)
- Reference: [events](reference/events.md), [configuration](reference/configuration.md),
  [filtering](reference/filtering.md), [async](reference/async.md)
- [PERFORMANCE.md](PERFORMANCE.md), [SECURITY_AUDIT.md](SECURITY_AUDIT.md),
  [TESTING.md](TESTING.md)
