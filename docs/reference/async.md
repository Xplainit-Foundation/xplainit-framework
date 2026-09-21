# Reference: Async Task Tracking

This page documents `AsyncTaskTracker`, grounded in
`xplainit-core/src/async_tracker.rs`. The module is plain data and logic: it
does **not** require the `async` cargo feature or a tokio runtime, so it builds
under default features.

## What it does

`AsyncTaskTracker` consumes a slice or iterator of `ExecutionEvent`s and
reconstructs, per async task id, the ordered lifecycle events plus the task's
current `TaskState`. It correctly separates interleaved events from multiple
concurrent tasks because timelines are keyed by `task_id`.

Only the three async variants are consumed (see [events.md](events.md)); all
other events are ignored:

- `AsyncTaskStart` — sets the task name, state `Running`, appends the event.
- `AsyncTaskAwait` — state `Awaiting`, appends the event.
- `AsyncTaskResume` — state `Running`, appends the event.

## `TaskState`

`Running`, `Awaiting`, `Completed`.

**Known limitation (documented in source):** there is no `AsyncTaskComplete`
event variant yet, because the language bindings that would emit async events
(PyO3/`asyncio`, the Node inspector, JVMTI, etc.) are not wired in this
environment. Introducing a completion event now would be speculative and
untested against a real runtime. Consequently, a task that runs to completion in
a captured trace ends in `Running` (last event was a resume) or `Awaiting` (it
suspended and the trace ended). `Completed` is only reached explicitly via
`mark_completed(task_id)`, for callers with out-of-band knowledge that a task
finished. When a real completion event is added, `record()` should map it here.

## `TaskTimeline`

```text
TaskTimeline {
    task_id: Uuid,
    task_name: Option<String>,   // captured from the start event when present
    events: Vec<ExecutionEvent>, // ordered lifecycle events
    state: TaskState,
}
```

Helpers: `len()`, `is_empty()`.

## `AsyncTaskTracker` API

- `new()` / `Default` — empty tracker.
- `from_events(&[ExecutionEvent]) -> Self` — build from a slice.
- `extend(iter)` — feed multiple events.
- `record(&ExecutionEvent)` — feed a single event (non-async ignored).
- `mark_completed(task_id)` — set a task to `Completed`.
- `task_ids() -> &[Uuid]` — all task ids in first-seen order.
- `task_count() -> usize`.
- `timeline(&task_id) -> Option<&TaskTimeline>`.
- `state_of(&task_id) -> Option<TaskState>`.
- `timelines() -> impl Iterator<Item = &TaskTimeline>` — in first-seen order.

## Example

```rust
use xplainit_core::AsyncTaskTracker;

// `events` is a decoded Vec<ExecutionEvent> (e.g. from a trace JSON).
let tracker = AsyncTaskTracker::from_events(&events);
for timeline in tracker.timelines() {
    println!(
        "task {} ({:?}): {} events, state {:?}",
        timeline.task_id,
        timeline.task_name,
        timeline.len(),
        timeline.state
    );
}
```

See the [async tracing guide](../guide/advanced/async-tracing.md) for the
end-user perspective.
