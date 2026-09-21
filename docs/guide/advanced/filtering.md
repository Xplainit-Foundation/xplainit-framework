# Advanced: Filtering

Filtering is how you keep traces focused and cheap. The framework offers two
layers; the precise, source-grounded API is in the
[filtering reference](../../reference/filtering.md). This page is the practical
overview.

## The `EventFilter` trait

Composable filters implement:

```rust
fn should_capture(&self, event: &ExecutionEvent, config: &Config) -> bool;
fn description(&self) -> String;
```

Built-in filters: `AcceptAllFilter`, `FunctionFilter`, `EventTypeFilter`,
`DepthFilter`, `CompositeFilter`, `TimeRangeFilter`, `ValueFilter`,
`FrequencyFilter`, `PathFilter`, and `UserFilter` (arbitrary predicate).

### Combine with `CompositeFilter`

```rust
use xplainit_core::{CompositeFilter, EventTypeFilter, PathFilter, EventFilter};

let filter = CompositeFilter::new(true) // require ALL to pass (AND)
    .add_filter(Box::new(EventTypeFilter::only_errors()))
    .add_filter(Box::new(PathFilter::new().exclude("site-packages")));
```

`require_all = false` switches to OR semantics. An empty composite keeps
everything.

### Sample noisy streams but keep errors

```rust
use xplainit_core::FrequencyFilter;

// Keep 1 in 100 events, but never drop errors or a hot event type.
let filter = FrequencyFilter::new(100)
    .always_keep_errors(true)
    .always_keep_type("function_enter");
```

### Match on values

```rust
use xplainit_core::ValueFilter;

// Only events whose value is a string containing "token"
let filter = ValueFilter::new().with_type("string").with_substring("token");
```

`pass_without_value(false)` drops events that carry no inspectable value.

## Advanced filters (`AdvancedFilter`)

For scope control by module/regex/depth/hot-path, use the standalone helpers
combined by `AdvancedFilter`:

```rust
use xplainit_core::{AdvancedFilter, ModuleFilter, RegexFilter, CallStackFilter};

let mut filter = AdvancedFilter::new()
    .with_module_filter(ModuleFilter::new().exclude_module("tests"))
    .with_regex_filter(RegexFilter::new().exclude_pattern(r"^_").unwrap())
    .with_call_stack_filter(CallStackFilter::new(10)); // max depth 10

let keep = filter.should_capture(&event, "thread-1");
```

`AdvancedFilter::should_capture` applies module → regex → depth → performance
(hot events are skipped, then sampling) in that order. `ModuleFilter` excludes
common stdlib paths by default (`exclude_stdlib = true`).

## Practical guidance

- To reduce overhead, filter as early and as narrowly as possible: exclude
  stdlib/vendor paths, restrict to the modules you care about, and cap depth.
- For Python full tracing, remember filtering reduces what is *recorded*, but
  the per-event `sys.settrace` callback still fires — see
  [performance tuning](performance-tuning.md).
