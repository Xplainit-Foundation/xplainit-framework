# Advanced: Async Tracing

Xplainit models async execution with three events (`AsyncTaskStart`,
`AsyncTaskAwait`, `AsyncTaskResume`) and reconstructs per-task timelines with
`AsyncTaskTracker`. The precise API is in the
[async reference](../../reference/async.md); this page is the practical view.

## Reconstructing task timelines

Given a decoded `Vec<ExecutionEvent>` (for example loaded from a trace JSON),
build a tracker and inspect each task:

```rust
use xplainit_core::{AsyncTaskTracker, TaskState};

let tracker = AsyncTaskTracker::from_events(&events);

println!("{} async task(s)", tracker.task_count());
for timeline in tracker.timelines() {
    println!(
        "task {} ({:?}): {} lifecycle events, state {:?}",
        timeline.task_id,
        timeline.task_name,
        timeline.len(),
        timeline.state,
    );
}
```

Interleaved events from multiple concurrent tasks are separated correctly
because timelines are keyed by `task_id`. Non-async events are ignored. The
tracker builds under default features (no `async` feature or tokio runtime
required) since it is plain data + logic.

## The completion caveat (important)

There is **no `AsyncTaskComplete` event variant**, on purpose: the language
bindings that would emit async events (PyO3/`asyncio`, the Node inspector,
JVMTI) are not wired in this environment, so adding a completion event now would
be speculative and untested against a real runtime.

Consequently, from events alone a task ends in:

- `TaskState::Running` — its last observed event was a resume, or
- `TaskState::Awaiting` — it suspended and the trace ended there.

`TaskState::Completed` is only reached explicitly via
`AsyncTaskTracker::mark_completed(task_id)`, for callers with out-of-band
knowledge that a task finished. Downstream views (CLI `tasks`, dashboard
`/tasks`) therefore surface `Running`/`Awaiting` honestly rather than guessing
completion. When a real completion event is added, `record()` should map it to
`Completed` so completion is derived from the trace directly.

## In the dashboard

The dashboard exposes a `/tasks` route that presents the reconstructed async
task view for a loaded trace, alongside the timeline and call graph.
