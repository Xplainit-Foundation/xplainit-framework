# Advanced: Custom Processors

Processors sit between filters and sinks in the pipeline. Where a filter answers
"keep this event?", a processor can transform, enrich, drop, or rate-limit
events as they flow through.

## The `EventProcessor` trait

Grounded in `xplainit-core/src/processor.rs`:

```rust
pub trait EventProcessor: Send + Sync {
    /// Process an event. Return:
    /// - Ok(Some(event)) to pass it on (possibly modified),
    /// - Ok(None) to drop it,
    /// - Err(..) on a processing error.
    fn process(&mut self, event: ExecutionEvent) -> Result<Option<ExecutionEvent>>;

    fn description(&self) -> String;
}
```

## Built-in processors

- **`PassThroughProcessor`** — no-op; useful for benchmarking.
- **`EnrichmentProcessor`** — attaches metadata to events.
- **`DeduplicationProcessor`** — removes duplicate events using an LRU-style
  cache.
- **`RateLimitProcessor`** — limits events per second.
- **`ProcessorPipeline`** — chains processors; its own `process()` runs each in
  turn and stops early if one drops the event (`Ok(None)`).

## Writing your own

Implement the trait and return `Ok(None)` to drop or `Ok(Some(event))` to
forward:

```rust
use xplainit_core::{EventProcessor, ExecutionEvent, Result};

struct KeepErrorsOnly;

impl EventProcessor for KeepErrorsOnly {
    fn process(&mut self, event: ExecutionEvent) -> Result<Option<ExecutionEvent>> {
        if event.is_error() {
            Ok(Some(event))
        } else {
            Ok(None) // drop non-errors
        }
    }

    fn description(&self) -> String {
        "Keep only error events".to_string()
    }
}
```

Chain it in a `ProcessorPipeline` (the `add_processor` builder consumes and
returns `self`; `DeduplicationProcessor::new` takes a max cache size):

```rust
use xplainit_core::{ProcessorPipeline, DeduplicationProcessor};

let mut pipeline = ProcessorPipeline::new()
    .add_processor(Box::new(DeduplicationProcessor::new(1024)))
    .add_processor(Box::new(KeepErrorsOnly));

if let Some(out) = pipeline.process(event)? {
    // forward `out` to a sink
}
```

`ProcessorPipeline::process` runs each processor in turn and stops early if one
returns `Ok(None)` (the event is dropped).

## Filter vs processor: which to use?

- Use a **filter** for cheap, side-effect-free "keep/drop" decisions
  (`&self`, no mutation).
- Use a **processor** when you need to mutate the event, maintain state across
  events (dedup, rate limiting), or return errors.
