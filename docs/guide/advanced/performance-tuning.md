# Advanced: Performance Tuning

This page is the practical companion to [`../../PERFORMANCE.md`](../../PERFORMANCE.md),
which records the FEAT-003 measurements. Read both; the numbers here are honest,
not aspirational.

## The headline reality: Python full tracing is expensive

Installing a global `sys.settrace()` hook fires a Python-level callback on
**every** call/return event. On a call-heavy microbenchmark (`fib(24)`,
best-of-3), measured overhead was approximately:

| Mode | Overhead vs untraced |
| --- | --- |
| Full call tracing | **~1200x** |
| Sampled (rate 0.05) | ~1160x (barely different) |

**The PRODUCTION plan's `<10%` overhead goal is NOT achievable for full
call-granularity tracing.** Sampling only reduces how many events are *recorded*
and cross into Rust; the per-event Python callback still fires, so it hardly
moves the ratio. This is inherent to the CPython `settrace` mechanism, not a
framework defect.

### How to actually get low overhead

Trace **selectively** instead of installing a global hook:

- Use the `@trace` decorators on hand-picked functions (no global hook).
- Narrow scope with `include_modules={...}` to your own modules.
- Use a shallow `max_depth`.
- Keep line tracing off (the default; no per-line hook installed).
- Exclude stdlib/vendor paths and sample noisy loops with filters (see
  [filtering](filtering.md)).

## Rust core hot-path work

FEAT-003 removed unnecessary clones/allocations on the hot path with no change
to public APIs or serialized JSON output (guarded by a golden test). Measured on
N = 200,000 events:

| Stage | Change |
| --- | --- |
| `PathFilter::should_capture` | ~1.9x faster (uses `location_ref()` instead of cloning `location()`) |
| `MemorySink::write` overflow eviction | ~28x faster (`VecDeque::pop_front()` O(1) instead of `Vec::remove(0)` O(n)) |
| `OutputFormatter::format_events` | writes into one pre-sized `String` (byte-identical output) |
| `event_store.record`, `pipeline.handle_event`, `json.format_events` | within run-to-run noise (their cost is dominated by work that was not changed) |

Speculative concurrency (extra lock-free structures, `crossbeam` channels) was
**declined**: the event store already uses a lock-free queue, and the measured
end-to-end cost is dominated by the language-binding side, so adding complexity
had no measured payoff.

## Error recovery and the `panic = "abort"` profile

The release profile sets `panic = "abort"`, so `std::panic::catch_unwind`
(used in `safe_execute`) **cannot** catch panics in release builds — the process
aborts. This is a deliberate, documented tradeoff. The profile-independent
recovery mechanism is therefore the **circuit-breaker** in `RuntimeControl`:

- Framework code routes recoverable failures through `record_error()`.
- After `max_consecutive_errors` (default 5; `0` disables) consecutive errors,
  the breaker trips (`enter_panic_mode`) and auto-disables tracing so a
  misbehaving trace target degrades the framework, not the host.
- `record_success()` resets the consecutive counter; telemetry
  (`total_errors`, `total_panics`, `times_tripped`) is always maintained.
- `debug_mode` (or `XPLAINIT_DEBUG`) logs framework errors to stderr; otherwise
  they are counted and swallowed.

Rely on `catch_unwind` only where unwinding is enabled (debug/test); rely on the
circuit-breaker everywhere.

## Measuring yourself

```sh
# Rust core micro-throughput (std-only harness, no external profiler):
cargo run --release --example bench_pipeline -p xplainit-core

# Python end-to-end overhead:
source .venv/bin/activate
cd xplainit-python && maturin develop --release && python bench_overhead.py
```

> `cargo-flamegraph` is unavailable offline, so profiling was qualitative plus
> the std harness above. No flamegraph output is fabricated.
