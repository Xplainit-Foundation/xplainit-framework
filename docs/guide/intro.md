# Introduction

Xplainit observes a program as it runs and produces a structured, explainable
record of what happened: which functions were entered and exited, how variables
changed, which branches and loops were taken, what errors occurred, and how
async tasks progressed. That record is a list of `ExecutionEvent` values (see
the [events reference](../reference/events.md)) which can be filtered, processed,
rendered as text/JSON/HTML/Markdown, or visualized in the dashboard.

## What Xplainit is good at today

- **A solid Rust core.** Event model, filtering, a filter → processor → sink
  pipeline, formatters, an event store, error analysis, and async task
  reconstruction are implemented and tested in `xplainit-core`.
- **Python runtime tracing** via `sys.settrace()` (the `xplainit` Python module
  built with maturin). Automatic tracing captures real user-code events; frame
  filtering and depth accounting are tested.
- **A CLI** for working with captured traces: `analyze`, `report`, and
  trace-driven `explain`.
- **A web dashboard** that loads a trace JSON and serves a timeline, call graph,
  and summary over fixed static routes.
- **Security and robustness hardening (Phase 4):** secret redaction on by
  default, input-path validation, an error-recovery circuit-breaker, and an
  extensive Rust + Python test suite.

## Honest limitations (read this before you rely on it)

Xplainit's own status documents historically overstated completeness. The single
source of truth is [`../STATUS.md`](../STATUS.md). The most important caveats:

- **Live tracing of an arbitrary program from the CLI is not wired.** The CLI
  `run` command does **not** attach a live tracer to a `.py`/`.js`/`.rs`/... file
  and trace it. Given a `.json` trace it renders it; given a source file it tells
  you exactly which binding to use to produce a trace. The `attach` style of
  live `explain` is likewise not wired; `explain --trace <file.json>` works on a
  captured trace. This is deliberate honesty in the code, not a bug.
- **Python full-tracing overhead is very high, not `<10%`.** Installing a global
  `sys.settrace()` hook fires a Python callback on every call/return, which
  dominates cost. Measured full-tracing overhead is roughly **~1200x** on a
  call-heavy microbenchmark; the `<10%` goal is **not achievable for full
  tracing** and is only approachable by tracing selectively (narrow modules,
  shallow depth, decorators). See [performance tuning](advanced/performance-tuning.md)
  and [`../PERFORMANCE.md`](../PERFORMANCE.md).
- **Non-Python bindings vary in maturity.** The C, Java, Go, and Node layers
  expose enable/disable/stats-style APIs; they are not full live source tracers,
  and Node cannot be rebuilt in the offline environment (no npm). Treat the
  language guides as accurate to the API surface, not as promises of full
  automatic tracing.
- **Some tooling is blocked offline.** `cargo audit`, `cargo tarpaulin`,
  `cargo-fuzz`, `mdbook`, and `cargo-flamegraph` are unavailable in the
  `INTEGRATIONS_ONLY` sandbox; the docs state where and provide the commands to
  run when online rather than fabricating results.

With those caveats understood, the [quickstart](getting-started/quickstart.md)
shows the paths that genuinely work today.
