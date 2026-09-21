# Concepts

A quick mental model of the pieces you will meet.

## Events

Everything Xplainit observes is an `ExecutionEvent`: function enter/exit,
variable declaration/assignment, conditionals, loops, returns, a rich set of
error variants, special detections (infinite loop, deadlock, memory leak), and
async task start/await/resume. Each event carries an `id`, a `timestamp`, and
usually a `SourceLocation`. Runtime values are captured as `Value` (null, bool,
integer, float, string, array, object, function, unknown). Full details in the
[events reference](../reference/events.md).

## Config

`Config` holds all knobs: language, verbosity, output format/destination/mode,
what to track (variables, calls, control flow, errors), depth, include/exclude
lists, and the Phase 4 additions — secret redaction (`redact_secrets`,
`redact_key_patterns`) and error-recovery (`max_consecutive_errors`,
`debug_mode`). Defaults are safe (redaction on, breaker at 5). It can be built
programmatically or loaded from `XPLAINIT_*` environment variables. See the
[configuration reference](../reference/configuration.md).

## Runtime control

`RuntimeControl` is the atomic enable/disable + recovery mechanism. It gates
capture/explain/error-tracking with fast atomic reads, rate-limits events, and
implements the **circuit-breaker**: after `max_consecutive_errors` consecutive
framework errors it trips (`enter_panic_mode`) and auto-disables tracing so a
misbehaving trace target degrades the framework, never the host. It also tracks
telemetry (`total_errors`, `total_panics`, `times_tripped`).

> The release profile uses `panic = "abort"`, so `catch_unwind` in
> `safe_execute` cannot catch panics in release builds. The circuit-breaker is
> the profile-independent recovery mechanism. See
> [performance tuning](../advanced/performance-tuning.md).

## The pipeline: filter → processor → sink

Captured events flow through a pipeline:

1. **Filters** (`EventFilter`) decide which events to keep. See
   [filtering](../advanced/filtering.md).
2. **Processors** (`EventProcessor`) transform, enrich, deduplicate, or
   rate-limit the stream. See [custom processors](../advanced/custom-processors.md).
3. **Sinks** (`EventSink`) receive the surviving events: console, file, memory,
   or a fan-out `MultiSink`.

An `EventStore` provides bounded (circular) storage with statistics.

## Explanations and formatters

`ExplanationGenerator` and `ErrorExplainer` turn events into natural-language
descriptions and error analysis. `OutputFormatter` implementations render events
as text, JSON, HTML, or Markdown — the same formats the CLI `report` command
exposes.

## Redaction

Before events leave the process (console, JSON, dashboard), values whose key
looks secret-like are replaced with `<redacted>`. This is on by default and
covers nested object/array payloads. See the
[configuration reference](../reference/configuration.md) and
[events reference](../reference/events.md).
