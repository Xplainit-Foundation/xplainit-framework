# Performance

This document records the FEAT-003 (Task 4.3) performance work: what was
optimized on the Rust hot event path, how it was measured, and the honest
numbers, including the parts of the plan's target that are **not** achievable.

All numbers below are wall-clock, single-machine measurements taken in the
offline development sandbox (release build, `lto=true`, `codegen-units=1`).
They are relative before/after signals, not absolute guarantees; absolute
throughput will differ on other hardware.

## Measurement method

Two independent harnesses are used, both offline and dependency-free:

1. **Rust core micro-throughput** — `xplainit-core/examples/bench_pipeline.rs`,
   a `std::time::Instant` harness (no `criterion`, no external profiler). It
   builds `N = 200_000` events and times each hot-path stage, reporting
   events/sec.

   ```sh
   cargo run --release --example bench_pipeline -p xplainit-core
   ```

   > A `criterion` bench (`xplainit-core/benches/event_capture.rs`) also
   > exists and is resolvable from `Cargo.lock`, but the std example is the
   > canonical offline harness for this task because it is guaranteed to run
   > with zero network access.

2. **Python end-to-end tracing overhead** — `xplainit-python/bench_overhead.py`,
   which times a CPU-bound, call-heavy workload (`fib(24)`) with tracing
   disabled vs enabled.

   ```sh
   source .venv/bin/activate
   cd xplainit-python && maturin develop --release && python bench_overhead.py
   ```

> Note on profiling tools: `cargo-flamegraph` is **unavailable** in this
> offline sandbox, so profiling was done qualitatively (reading the hot path)
> plus the quantitative std harness above. No flamegraph output is fabricated.

## What was optimized

The hot path (`pipeline.rs` → `processor.rs` → `sink.rs`, plus `filter.rs`,
`advanced_filter.rs`, `formatter.rs`, `events.rs`) was reviewed for
unnecessary clones and allocations. Behavior, public APIs, and serialized JSON
output are unchanged (see the golden test in
`xplainit-core/tests/perf_regression.rs`).

| Change | File | Effect |
| --- | --- | --- |
| Added `ExecutionEvent::location_ref() -> Option<&SourceLocation>` and used it on the filter path instead of the cloning `location()`. | `events.rs`, `filter.rs`, `advanced_filter.rs` | Removes a per-event `SourceLocation` clone (owns a `String` path) on every filter check. Owned `location()` is kept for API compatibility. |
| `MemorySink` now stores events in a `VecDeque` and evicts the oldest with `pop_front()` instead of `Vec::remove(0)`. | `sink.rs` | Overflow eviction goes from O(n) element-shift to O(1). Same FIFO ordering and same `Vec<ExecutionEvent>` returned by `get_events()`. |
| `OutputFormatter::format_events` writes into one pre-sized `String` instead of collecting a `Vec<String>` and `join`-ing. | `formatter.rs` | Removes intermediate per-event `String` vector allocation; output is byte-identical to the previous `"\n"`-join. |

## Measured results (Rust core, N = 200,000 events)

| Stage | Before (events/sec) | After (events/sec) | Change |
| --- | ---: | ---: | ---: |
| `path_filter.should_capture` | 21,778,870 | 41,658,483 | ~1.9x faster |
| `memory_sink.write` (overflow, cap 1000) | 256,234 | 7,325,924 | ~28x faster |
| `event_store.record` | 3,489,509 | 3,344,521 | within noise (unchanged) |
| `pipeline.handle_event` | 2,244,603 | 2,080,331 | within noise (unchanged) |
| `json.format_events` | 2,363,334 | 2,471,752 | within noise |

The two targeted stages improved clearly. The `event_store`, `pipeline`, and
`json` stages are dominated by work that was **not** changed (lock-free queue
push, `serde_json` serialization), so their numbers move only within run-to-run
noise — which is the expected, honest result.

## Concurrency / lock-free evaluation (declined)

The plan asked whether the event store/pipeline should adopt additional
lock-free or parallel processing (e.g. `crossbeam` channels).

**Decision: declined, with rationale.**

- `EventStore` already uses a lock-free `crossbeam::queue::ArrayQueue` for the
  events themselves; only the small `EventStats` counters sit behind a
  `parking_lot::RwLock`. The `record()` hot path takes that write lock once per
  event. Replacing the stats lock with atomics is plausible, but the current
  bench shows `event_store.record` is not lock-bound at these volumes
  (~3.4M events/sec single-threaded), and the Python-side cost dwarfs it by
  ~1000x (see below). Adding atomics would be speculative micro-tuning with no
  measured payoff on the real end-to-end workload.
- A `crossbeam` channel between collection and sinking would add a thread and a
  hand-off cost without a demonstrated contention problem to solve; the
  pipeline is currently driven synchronously per event.

Per the task's guidance ("if it adds risk without measured benefit, document
the evaluation and decline"), no speculative concurrency was added. The
optimizations that shipped are the ones with a clear, test-covered win.

## Python end-to-end overhead (honest)

Measured with `bench_overhead.py`, `fib(24)`, best of 3 runs:

| Mode | Time | Overhead |
| --- | ---: | ---: |
| Untraced baseline | 8.008 ms | — |
| Full call tracing | 9648.284 ms | **~1204.8x (+120,384%)** |
| Sampled (`sampling_rate=0.05`) | 9316.855 ms | ~1163.5x |

### The `<10%` target is NOT achievable for full tracing

The PRODUCTION plan's "<10% overhead" goal is **not met and not achievable for
full call-granularity tracing**, and this document does not claim otherwise.
Once `sys.settrace()` is installed, a Python-level callback fires on **every**
call/return event; that alone dwarfs the traced workload before any Rust code
runs. Sampling only reduces how many events are *recorded* (and cross into
Rust) — the per-event Python callback still fires — so it barely moves the
ratio.

Low overhead is only reachable by **not** installing a global trace hook and
tracing selectively instead:

- `include_modules={...}` to narrow scope to specific user modules,
- a shallow `max_depth`,
- the `@trace` decorators for hand-picked functions,
- `trace_lines=False` (the default) so the per-line hook is never installed.

The Rust-side optimizations above make the *framework's own* per-event handling
cheaper, which is the part under this project's control; the dominant cost for
full tracing is the CPython `settrace` mechanism itself, which is inherent.
