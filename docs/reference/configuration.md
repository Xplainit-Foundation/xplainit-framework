# Reference: Configuration

This page documents `Config` and its related enums. Every field, default, and
environment variable below is grounded in `xplainit-core/src/config.rs` (and,
for the runtime-control mirror of some settings, `control.rs`).

## Enums

### `Language`
`Python`, `JavaScript`, `C`, `Cpp`, `Java`, `Go`, `Rust`.

- `Language::as_str()` returns the lowercase name (`python`, `javascript`, `c`,
  `cpp`, `java`, `go`, `rust`).
- `Language::parse(s)` is case-insensitive and accepts aliases: `py`, `js`/`node`,
  `c++`/`cxx`, `golang`, `rs`. Returns `None` for anything else.

### `Verbosity`
`Brief`, `Normal` (default), `Detailed`, `Debug`.

### `OutputFormat`
`Console`, `ConsoleColored` (default), `Json`, `Html`, `Markdown`.

### `OutputDestination`
`Stdout` (default), `Stderr`, `File(PathBuf)`, `Network(String)`,
`Multiple(Vec<OutputDestination>)`.

### `OutputMode`
`Streaming` (default), `Buffered`, `Manual`.

## `Config` fields and defaults

`Config::default()` (and `Config::new(language)`, which only overrides
`language`) produces:

| Field | Type | Default |
| --- | --- | --- |
| `language` | `Language` | `Python` |
| `verbosity` | `Verbosity` | `Normal` |
| `output_format` | `OutputFormat` | `ConsoleColored` |
| `output_destination` | `OutputDestination` | `Stdout` |
| `output_mode` | `OutputMode` | `Streaming` |
| `show_line_numbers` | `bool` | `true` |
| `show_source_code` | `bool` | `true` |
| `color_output` | `bool` | `true` |
| `max_depth` | `usize` | `100` |
| `track_variables` | `bool` | `true` |
| `track_function_calls` | `bool` | `true` |
| `track_control_flow` | `bool` | `true` |
| `capture_errors` | `bool` | `true` |
| `include_timestamps` | `bool` | `false` |
| `include_thread_info` | `bool` | `false` |
| `include_functions` | `Vec<String>` | empty |
| `exclude_functions` | `Vec<String>` | empty |
| `include_modules` | `Vec<String>` | empty |
| `exclude_modules` | `Vec<String>` | empty |
| `redact_secrets` | `bool` | `true` |
| `redact_key_patterns` | `Vec<String>` | default secret patterns (below) |
| `max_consecutive_errors` | `u64` | `5` |
| `debug_mode` | `bool` | `false` |

### Builder methods

`Config::new(language)`, then chainable: `with_verbosity`, `with_output_format`,
`with_output_destination`, `with_output_mode`, `with_max_depth`.

## Secret redaction settings (Phase 4, Task 4.1)

- **`redact_secrets` (default `true`)** — safe-by-default. When enabled, values
  whose key matches `redact_key_patterns` are replaced with the placeholder
  `<redacted>` before they leave the process (console output, JSON, dashboard).
- **`redact_key_patterns`** — case-insensitive **substrings** that mark a key as
  secret-like. Defaults come from
  `xplainit_core::security::DEFAULT_REDACTION_PATTERNS`:
  `password`, `passwd`, `secret`, `token`, `api_key`, `apikey`,
  `authorization`, `auth`, `credential`, `private_key`.

  Because matching is substring-based, keys like `db_password` or `user_token`
  are covered. Redaction is applied via `ExecutionEvent::redacted()` /
  `redact_events()`; see [events.md](events.md) for exactly which fields are
  scrubbed, including nested object/array payloads.

## Error-recovery settings (Phase 4, Task 4.2)

- **`max_consecutive_errors` (default `5`)** — the number of *consecutive*
  framework errors tolerated before the runtime circuit-breaker trips and
  auto-disables tracing. `0` disables the breaker entirely (errors are still
  counted for telemetry but never auto-disable tracing). This threshold is read
  by `RuntimeControl::new()` into an atomic and can be changed at runtime with
  `RuntimeControl::set_max_consecutive_errors()`.
- **`debug_mode` (default `false`)** — when `true`, framework-internal errors
  are logged to stderr with context; when `false`, they are counted for
  telemetry and swallowed so they never reach the host program. Also enabled at
  runtime via the `XPLAINIT_DEBUG` environment variable.

See [../reference/filtering.md](filtering.md) for filters and the async
reference for task tracking; the recovery mechanics live in `control.rs` and are
summarized in [../guide/advanced/performance-tuning.md](../guide/advanced/performance-tuning.md).

## `Config::from_env()`

Reads these environment variables (unrecognized values fall back to defaults):

| Variable | Effect |
| --- | --- |
| `XPLAINIT_LANGUAGE` | parsed via `Language::parse` |
| `XPLAINIT_VERBOSITY` | `brief`/`normal`/`detailed`/`debug` |
| `XPLAINIT_OUTPUT` | `stdout`, `stderr`, or a file path |
| `XPLAINIT_DEBUG` | truthy (anything but empty/`0`/`false`) enables `debug_mode` |
| `XPLAINIT_MAX_CONSECUTIVE_ERRORS` | parsed as `u64` into `max_consecutive_errors` |

`RuntimeControl::from_env()` additionally honors `XPLAINIT_ENABLED`,
`XPLAINIT_CAPTURE`, `XPLAINIT_EXPLAIN`, and `XPLAINIT_MAX_EVENTS_PER_SEC`.
