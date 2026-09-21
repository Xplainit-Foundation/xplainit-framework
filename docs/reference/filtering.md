# Reference: Filtering

The framework has two filtering layers, both grounded here in
`xplainit-core/src/filter.rs` and `xplainit-core/src/advanced_filter.rs`.

## The `EventFilter` trait (`filter.rs`)

```rust
pub trait EventFilter: Send + Sync {
    fn should_capture(&self, event: &ExecutionEvent, config: &Config) -> bool;
    fn description(&self) -> String;
}
```

`should_capture` returns `true` to keep an event. All the filters below
implement this trait, so they compose inside a `CompositeFilter`.

### `AcceptAllFilter`
Keeps every event.

### `FunctionFilter`
Filters by function name for `FunctionEnter`/`FunctionExit` events (other events
pass through). Fields: `include: HashSet<String>`, `exclude: HashSet<String>`,
`trace_stdlib: bool` (default `false`). Rules, in order:
1. Excluded names are dropped.
2. If `include` is non-empty, only listed names are kept.
3. If `trace_stdlib` is false, stdlib-looking names are dropped. A name is
   treated as stdlib if it starts with `std::`, `__`, or `_`, or contains
   `builtins`, `System.`, `java.lang`, or `java.util`.

Builders: `include(name)`, `exclude(name)`, `with_stdlib(bool)`.

### `EventTypeFilter`
Boolean toggles: `capture_normal`, `capture_errors`, `capture_functions`,
`capture_variables`, `capture_loops`. Constructors: `new()` (all true),
`only_errors()`, `only_functions()`. Function/variable/loop events are gated by
their toggle; error events (per `ExecutionEvent::is_error()`) by `capture_errors`;
everything else by `capture_normal`.

### `DepthFilter`
Drops `FunctionEnter` events once `current_depth >= max_depth`; all other events
pass. (Note: `current_depth` is a plain field on this filter and is not mutated
by `should_capture`, which takes `&self`; for stateful depth accounting use the
`CallStackFilter` in `advanced_filter.rs`.)

### `CompositeFilter`
Combines child filters. `new(require_all)`: when `require_all` is `true` ALL
child filters must pass (AND); when `false` ANY may pass (OR). An empty composite
keeps everything. Add children with `add_filter(Box<dyn EventFilter>)`.

### `TimeRangeFilter`
Inclusive `start`/`end` bounds, each `Option<DateTime<Utc>>` (open when `None`).
Constructors: `new()` (fully open), `with_start`, `with_end`, `between(start, end)`.
An event passes when its `timestamp()` is within the closed window.

### `ValueFilter`
Inspects the values reachable from an event
(`VariableAssign` new/old, `VariableDeclaration`, `FunctionExit.return_value`,
`Return.value`, `TypeError.value`, `DivisionByZero.numerator`,
`LoopIteration.loop_var_value`, `FunctionEnter.args`). Fields:
- `substring: Option<String>` — matches when a rendered value contains it.
- `type_name: Option<String>` — matches when a value's `type_name()` equals it.
- `pass_without_value: bool` (default `true`) — events with no inspectable value
  pass or are dropped based on this flag.

An event with values matches if **any** value satisfies all set criteria.
Builders: `with_substring`, `with_type`, `pass_without_value`.

### `FrequencyFilter`
Deterministic 1-in-N sampling using an internal `AtomicUsize` (no `rand`
dependency). `new(rate)` clamps the rate to at least 1. Every N-th subject event
is captured. `always_keep_type(event_type)` and `always_keep_errors(true)` let
listed types / error events bypass sampling entirely.

### `PathFilter`
Filters by source file path. `include`/`exclude` are `Vec<String>` accepting
plain substrings or simple globs (`**` across separators, `*` within a segment,
`?` single char). Exclusions always win; with an empty include list any
non-excluded path passes. Uses `location_ref()` and the `<unknown>` sentinel for
locationless events.

### `UserFilter`
Wraps a user predicate `Fn(&ExecutionEvent, &Config) -> bool + Send + Sync`.
`new(predicate)` or `with_label(label, predicate)` for a descriptive
`description()`.

## Advanced filters (`advanced_filter.rs`)

These are standalone helpers (not `EventFilter` implementers) combined by
`AdvancedFilter`.

### `ModuleFilter`
`include_modules`/`exclude_modules` (`HashSet<String>`), `exclude_stdlib: bool`
(default `true`), `exclude_patterns: Vec<String>` (simple globs).
`should_filter_file(path) -> bool` returns `true` to filter **out**. Built-in
stdlib patterns include `site-packages`, `/lib/python`, `/lib64/python`,
`\Lib\`, `node_modules`, `/usr/lib`, `/usr/local/lib`, `C:\Windows\System32`.
Builders: `include_module`, `exclude_module`, `exclude_pattern`.

### `RegexFilter`
`include_pattern`/`exclude_pattern` compile `regex::Regex` (returning
`Result<Self, regex::Error>`). `should_include_function(name)`: exclusions win;
if any include patterns exist at least one must match; otherwise allow.

### `CallStackFilter`
Stateful per-thread depth accounting. `new(max_depth)` (`0` = unlimited).
`should_capture_at_depth(thread_id, event)` increments on `FunctionEnter`
(rejecting once depth exceeds `max_depth`), decrements on `FunctionExit` (always
kept to keep the stack balanced), and gates other events by current depth.
`reset_thread`, `current_depth` also provided.

### `PerformanceFilter`
`mark_hot_function`, `mark_hot_path`, `with_sampling(rate)`,
`exclude_loop_bodies()`. `is_hot_event(event)` flags events in hot functions /
paths (and loop iterations when loops are excluded); `should_sample()` performs
1-in-N sampling.

### `AdvancedFilter`
Combines the four above. `should_capture(event, thread_id)` applies, in order:
module filter, function-name regex filter, call-stack depth filter, then the
performance filter — where hot events are **skipped** and sampling is applied.
Builders: `with_module_filter`, `with_regex_filter`, `with_call_stack_filter`,
`with_performance_filter`.
