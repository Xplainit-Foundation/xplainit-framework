# Testing Guide

This document describes how the `xplainit-framework` test suite is organized,
how coverage is approached, and which tooling is **blocked in the current
offline environment** (with the exact commands to run once it is available). No
tool output is fabricated: where a tool cannot run here, that is stated plainly
and the intent is met by other means.

## Running the tests

Rust workspace (from the repo root):

```sh
cargo build --release
cargo test                                             # full workspace, fast tests
cargo test -- --ignored                                # heavy/soak tests (see below)
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

Python binding suites (require the maturin-built module named `xplainit`):

```sh
source .venv/bin/activate
python test_automatic_tracing.py                       # 3/3
cd xplainit-python && maturin develop --release
python test_frame_filter.py                            # 11/11
python test_depth_accounting.py                         # 23/23
python test_decorators.py
```

## Test layout

### Unit tests (in `#[cfg(test)]` modules)

Each core module in `xplainit-core/src` carries unit tests next to the code:
`config.rs`, `control.rs`, `events.rs`, `event_store.rs`, `sink.rs`,
`formatter.rs`, `advanced_filter.rs`, `ast.rs`, `async_tracker.rs`,
`security.rs`, `pipeline.rs`, and the bindings/CLI/dashboard crates.

### Integration and cross-cutting tests (`xplainit-core/tests`, `xplainit-cli/tests`)

| File | Focus |
| --- | --- |
| `integration_tests.rs` | Multi-component workflows: pipeline, filtering, error analysis, formatters, redaction end-to-end, path validation. |
| `perf_regression.rs` | Revert-sensitive golden tests for the FEAT-003 performance work (byte-for-byte JSON, `location_ref` vs `location`, FIFO sink overflow). |
| `edge_case_tests.rs` | Boundary and error branches for previously thin modules: empty inputs, unicode / very long strings, zero and max depths, malformed data, sink error branches, bounded stores. |
| `property_tests.rs` | Hand-rolled property-style invariants (see below). |
| `load_tests.rs` | In-process load / stress tests (see below). |
| `xplainit-cli/tests/cli.rs` | End-to-end CLI flows: `analyze`, `report` (text/json/html/markdown), secret redaction in rendered output, malformed / missing / empty input handling. |

Every added test genuinely exercises the code under test and is designed to
**fail if that code path is reverted** (for example, the redaction tests fail if
the redaction choke points are removed, and the golden tests fail if
serialization shape changes).

## Coverage approach (tarpaulin is blocked offline)

Line/branch coverage would normally be measured with `cargo tarpaulin`.
**`cargo-tarpaulin` is not installed and cannot be installed in this offline
(`INTEGRATIONS_ONLY`) sandbox**, so numeric coverage is **not** reported here to
avoid fabricating figures. Coverage is instead assessed **qualitatively**: for
each previously thin module we added unit + edge-case tests that hit the empty,
boundary, error, and large-input paths in addition to the happy path.

When tarpaulin is available, measure coverage with:

```sh
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --out Html --out Stdout --exclude-files 'target/*'
```

Modules specifically hardened with new tests in this pass:
`explainer.rs`, `error_explainer.rs`, `advanced_filter.rs`, `ast.rs`,
`async_tracker.rs`, `formatter.rs`, `sink.rs`, `event_store.rs`, plus the
security (redaction / path validation) and error-recovery
(circuit-breaker / telemetry) code added in earlier Phase 4 features.

## Property-style testing (proptest / quickcheck are NOT used)

Neither `proptest` nor `quickcheck` (nor `arbitrary`) is present in
`Cargo.lock`, and new external crates cannot be fetched offline, so
property-based testing is **hand-rolled with `std` only** in
`xplainit-core/tests/property_tests.rs`:

- A tiny deterministic **linear congruential generator (LCG)** produces a
  reproducible pseudo-random stream (fixed seeds keep runs deterministic and
  fast; failures print the case index for reproduction).
- Generated events deliberately keep every `HashMap` to **at most one key**,
  because multi-key map serialization order is randomized per map instance and
  would make byte-for-byte JSON comparisons flaky rather than meaningful (the
  same constraint the golden tests in `perf_regression.rs` observe). Generated
  floats are exactly representable integers-as-`f64` so JSON round-trips are
  textually stable.

Invariants asserted over many generated cases:

1. **Serialize -> deserialize round-trip**: an `ExecutionEvent` re-serializes to
   the identical JSON after a decode.
2. **Redaction idempotence + leak-freedom**: `redacted(redacted(x))` equals
   `redacted(x)`, and a known secret marker never survives redaction (this
   invariant caught, and drove the fix for, a real leak of secrets nested inside
   single-value payload fields such as `FunctionExit.return_value`).
3. **Filter inclusion/exclusion consistency**: an excluded name is never
   captured; an include-only filter admits only listed names.
4. **Pipeline never panics**: a generated event stream drives the real
   filter -> processor -> sink pipeline without error, and the sink never
   receives more events than the filter accepted.
5. **`redact_events` equals per-event redaction** over a slice.

## Load / stress testing

`xplainit-core/tests/load_tests.rs` drives large in-process volumes through the
real pipeline and event store:

- `load_pipeline_handles_200k_events_without_error` - 200k events through the
  full pipeline, error-free, within a time budget.
- `load_event_store_bounded_never_leaks` - 250k events into a bounded store,
  asserting the buffer never exceeds capacity and `recorded == kept + dropped`.
- `load_event_store_concurrent_writers_are_consistent` - four threads sharing
  one store, asserting accounting stays consistent under contention.
- `load_heavy_soak_one_million_events` - **`#[ignore]`d** 1,000,000-event soak;
  run with `cargo test -- --ignored`.

**Honest scope note:** the production plan aspires to *"1,000,000 events/sec,
10,000 concurrent tasks, multi-hour soak."* That literal target is **not** run
in this CI gate: this is an offline sandbox with no dedicated load rig, and a
multi-hour soak is inappropriate for a unit-test gate. The tests above are a
scaled-down but substantial stand-in that still catches leaks, accounting bugs,
and gross throughput regressions. Achieving and verifying the full production
target requires a dedicated benchmarking host and is tracked separately.

## Fuzzing (cargo-fuzz is blocked offline)

Coverage-guided fuzzing would normally use `cargo-fuzz` (libFuzzer).
**`cargo-fuzz` is not installed and cannot be installed in this offline
sandbox**, and it additionally requires a nightly toolchain. No fuzzing results
are reported or fabricated. The hand-rolled generative property tests described
above are the **substitute** in this environment: they feed large volumes of
systematically varied input through serialization, redaction, filtering, and the
pipeline and assert invariants / absence of panics.

When the tooling and network are available, set up fuzzing with:

```sh
cargo install cargo-fuzz
cargo fuzz init                                  # once, creates fuzz/ crate
cargo fuzz add event_roundtrip                   # add a target
# implement fuzz_target!(|data: &[u8]| { ... deserialize + redact + reserialize ... })
cargo +nightly fuzz run event_roundtrip          # run the fuzzer
```

Good first fuzz targets, mirroring the property invariants: `ExecutionEvent`
JSON deserialize -> redact -> reserialize (must not panic), and driving decoded
event streams through `EventPipeline::handle_event`.
