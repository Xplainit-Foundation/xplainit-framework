# Security Audit (Dependency & Supply-Chain)

This document records the dependency-audit approach for the Xplainit framework
as part of Phase 4 (Production Hardening, Task 4.1). It is written honestly for
an **offline build environment**: it states which automated tooling is blocked
here and gives the exact commands a maintainer should run once network access is
available. No advisory results are fabricated in this file.

## 1. Environment constraint

The audit was prepared inside an offline sandbox with **repository access only**
(no access to crates.io, the RustSec advisory database, or other external
registries). As a direct consequence the following tools **cannot be installed
or run here**, so their output is **not** included in this document:

- `cargo audit` (RustSec advisory scanning) — requires downloading the advisory
  database from `https://github.com/RustSec/advisory-db`.
- `cargo-vet` (supply-chain attestation) — requires fetching audit records and
  registry metadata.
- `cargo outdated` (newer-version detection) — requires querying crates.io.

These are not skipped as unnecessary; they are simply unavailable in this
environment. The section below lists exactly what to run when online.

## 2. What was done by available means (offline)

- **Version pinning via `Cargo.lock`.** The repository commits `Cargo.lock`, so
  every transitive dependency resolves to an exact, reproducible version. The
  lockfile currently pins **202** package entries, including the 7 workspace
  member crates (count from `grep -c '^name = ' Cargo.lock`). Builds therefore
  do not silently float onto new upstream versions.
- **Direct dependency review.** The workspace declares a deliberately small set
  of well-established, widely-audited direct dependencies (see below). No new
  external crates were added during this task; all hardening work uses `std`
  plus dependencies already present in `Cargo.lock`.
- **FFI / `unsafe` audit.** Independently of dependency scanning, every `unsafe`
  block in the C (`xplainit-c`) and Java (`xplainit-java`) bindings now carries a
  `// SAFETY:` comment documenting its invariants, and raw-pointer entry points
  guard against null pointers before dereferencing. See those crates' sources.
- **Input validation.** User-supplied trace/log paths are validated
  (`xplainit_core::validate_input_path`) before being opened.
- **Secret/PII redaction.** Captured runtime values whose key matches a
  secret-like pattern are redacted before leaving the process
  (`xplainit_core::redact_events`, config `redact_secrets` / `redact_key_patterns`).
  Redaction is primarily *key-based* (it keys off the name of an argument,
  variable or context entry), and it also recurses into nested object/array
  values so a secret nested inside a return value, resumed value or loop
  variable is scrubbed too.
- **Error/exception message scrubbing.** Free-form `message` strings on
  `Exception`, `RuntimeError`, `SyntaxError` and `Panic` carry no per-value key,
  so key-based redaction cannot reach a secret pasted inline (for example
  `"auth failed: token=sk-..."` or a connection string with an embedded
  password). These messages are additionally scanned with
  `xplainit_core::security::redact_message`, which finds a redaction pattern
  immediately followed by a `=`/`:` (optionally `=>`) assignment and replaces
  the value token with the placeholder, leaving the surrounding prose intact.
  Limitation: this scrubs `key=value` / `key: value` assignments; a secret that
  appears in a message with no adjacent key (a bare token in free prose) is not
  detectable by name and is not redacted.
- **Configured redaction policy on load paths.** The CLI and dashboard
  `load_events` load their redaction policy from the environment via
  `Config::from_env` (respecting `XPLAINIT_REDACT_SECRETS` and a comma-separated
  `XPLAINIT_REDACT_KEY_PATTERNS`), so customized patterns and `XPLAINIT_*`
  settings actually take effect where events leave the process instead of a
  hardcoded default. Both also expose `load_events_with_config` for callers that
  thread an explicit `Config`.

## 3. Direct workspace dependencies (from `Cargo.toml`)

Shared workspace dependencies (`[workspace.dependencies]`):

| Crate         | Pinned constraint | Purpose                         |
| ------------- | ----------------- | ------------------------------- |
| serde         | 1.0               | Serialization                   |
| serde_json    | 1.0               | JSON encode/decode              |
| thiserror     | 1.0               | Error type derivation           |
| anyhow        | 1.0               | CLI/dashboard error propagation |
| log           | 0.4               | Logging facade                  |
| env_logger    | 0.11              | Log backend                     |
| colored       | 2.1               | Colored console output          |
| termcolor     | 1.4               | Terminal color support          |
| tokio         | 1.35              | Async runtime (dashboard)       |
| async-trait   | 0.1               | Async trait support             |
| parking_lot   | 0.12              | Locks/atomics                   |
| crossbeam     | 0.8               | Concurrency primitives          |
| config        | 0.14              | Configuration loading           |
| toml          | 0.8               | TOML parsing                    |

Additional per-crate direct dependencies of note:

- `xplainit-c`: `uuid`, `chrono`, `serde_json`; build-dep `cbindgen`.
- `xplainit-java`: `jni`, `uuid`, `chrono`, `serde_json`.
- `xplainit-python`: `pyo3` (built via `maturin`).
- `xplainit-node`: `neon`.

Exact resolved versions for all transitive crates are authoritative in
`Cargo.lock`.

## 4. Commands to run when online

Run these from the repository root on a machine with network access. Record the
real output alongside this file (do not paste unverified results):

```sh
# Install the tooling (once):
cargo install cargo-audit
cargo install cargo-outdated
cargo install cargo-vet

# 1. Scan the committed lockfile against the RustSec advisory database.
cargo audit

# 2. Report dependencies with newer versions available.
cargo outdated --workspace --root-deps-only

# 3. (Optional) Supply-chain attestation review.
cargo vet
```

### Interpreting results

- `cargo audit` exits non-zero if any advisory matches a pinned version. Triage
  each finding: upgrade the affected crate (and re-commit `Cargo.lock`), or
  document an accepted risk with justification.
- `cargo outdated` is informational; prefer upgrading crates that also appear in
  `cargo audit` findings first.
- Re-run the full gate after any dependency change:
  `cargo build --release && cargo test && cargo clippy --all-targets --all-features -- -D warnings && cargo fmt --all -- --check`.

## 5. Maintenance cadence

- Re-run `cargo audit` in CI on every push once the environment has network
  access (the RustSec DB updates frequently).
- Re-review this document whenever a new direct dependency is added; keep the
  table in section 3 in sync with `Cargo.toml`.
