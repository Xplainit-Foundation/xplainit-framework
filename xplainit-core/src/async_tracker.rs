//! Async task lifecycle tracking.
//!
//! The [`AsyncTaskTracker`] consumes a slice or iterator of [`ExecutionEvent`]s
//! and reconstructs, per async task id, the ordered lifecycle events plus the
//! task's current [`TaskState`]. It correctly handles interleaved events coming
//! from multiple concurrent tasks, since tasks are keyed by their `task_id`.
//!
//! This module is plain data and logic: it does not require the `async` cargo
//! feature or a tokio runtime, so it builds under default features.

use crate::events::ExecutionEvent;
use std::collections::HashMap;
use uuid::Uuid;

/// The reconstructed state of an async task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// The task has started (or resumed) and is actively running.
    Running,
    /// The task is suspended at an await point.
    Awaiting,
    /// The task has completed.
    ///
    /// # Known limitation: not reachable from events alone
    ///
    /// The [`ExecutionEvent`] model intentionally has no `AsyncTaskComplete`
    /// variant yet: the language bindings that will emit async events
    /// (PyO3/`asyncio`, the Node inspector, etc.) are not wired in this
    /// environment, so introducing a completion event now would be speculative
    /// and untested against a real runtime. Consequently a task that runs to
    /// completion in a captured trace ends in [`TaskState::Running`] (its last
    /// observed event was a resume) or [`TaskState::Awaiting`] (it suspended and
    /// the trace ended there).
    ///
    /// This state is therefore only reached explicitly, via
    /// [`AsyncTaskTracker::mark_completed`], for callers that have out-of-band
    /// knowledge that a task finished. When a real completion event is added to
    /// [`ExecutionEvent`], [`AsyncTaskTracker::record`] should map it here so
    /// completion is derived from the trace directly. Until then, downstream
    /// views (CLI/dashboard) surface `Running`/`Awaiting` honestly rather than
    /// guessing completion.
    Completed,
}

/// The reconstructed timeline and state for a single async task.
#[derive(Debug, Clone)]
pub struct TaskTimeline {
    /// The task's unique id.
    pub task_id: Uuid,
    /// Human-readable task name, captured from the start event when available.
    pub task_name: Option<String>,
    /// The ordered lifecycle events for this task (start, awaits, resumes).
    pub events: Vec<ExecutionEvent>,
    /// The task's current state.
    pub state: TaskState,
}

impl TaskTimeline {
    fn new(task_id: Uuid) -> Self {
        Self {
            task_id,
            task_name: None,
            events: Vec::new(),
            state: TaskState::Running,
        }
    }

    /// Number of lifecycle events recorded for this task.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether this task has no recorded lifecycle events.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Reconstructs per-task async lifecycle timelines from a stream of events.
#[derive(Debug, Default)]
pub struct AsyncTaskTracker {
    // Preserve first-seen ordering of task ids for stable iteration.
    order: Vec<Uuid>,
    tasks: HashMap<Uuid, TaskTimeline>,
}

impl AsyncTaskTracker {
    /// Create an empty tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a tracker from a slice of events.
    pub fn from_events(events: &[ExecutionEvent]) -> Self {
        let mut tracker = Self::new();
        tracker.extend(events.iter());
        tracker
    }

    /// Feed multiple events into the tracker.
    pub fn extend<'a, I>(&mut self, events: I)
    where
        I: IntoIterator<Item = &'a ExecutionEvent>,
    {
        for event in events {
            self.record(event);
        }
    }

    /// Feed a single event into the tracker. Non-async events are ignored.
    pub fn record(&mut self, event: &ExecutionEvent) {
        match event {
            ExecutionEvent::AsyncTaskStart {
                task_id, task_name, ..
            } => {
                let timeline = self.timeline_mut(*task_id);
                timeline.task_name = Some(task_name.clone());
                timeline.state = TaskState::Running;
                timeline.events.push(event.clone());
            }
            ExecutionEvent::AsyncTaskAwait { task_id, .. } => {
                let timeline = self.timeline_mut(*task_id);
                timeline.state = TaskState::Awaiting;
                timeline.events.push(event.clone());
            }
            ExecutionEvent::AsyncTaskResume { task_id, .. } => {
                let timeline = self.timeline_mut(*task_id);
                timeline.state = TaskState::Running;
                timeline.events.push(event.clone());
            }
            _ => {}
        }
    }

    fn timeline_mut(&mut self, task_id: Uuid) -> &mut TaskTimeline {
        if !self.tasks.contains_key(&task_id) {
            self.order.push(task_id);
            self.tasks.insert(task_id, TaskTimeline::new(task_id));
        }
        self.tasks
            .get_mut(&task_id)
            .expect("timeline just inserted")
    }

    /// Mark a task as completed. Useful when a caller knows a task finished
    /// (there is no dedicated completion event variant yet).
    pub fn mark_completed(&mut self, task_id: Uuid) {
        if let Some(timeline) = self.tasks.get_mut(&task_id) {
            timeline.state = TaskState::Completed;
        }
    }

    /// All task ids seen, in first-seen order.
    pub fn task_ids(&self) -> &[Uuid] {
        &self.order
    }

    /// Number of tracked tasks.
    pub fn task_count(&self) -> usize {
        self.order.len()
    }

    /// The reconstructed timeline for a specific task.
    pub fn timeline(&self, task_id: &Uuid) -> Option<&TaskTimeline> {
        self.tasks.get(task_id)
    }

    /// The current state of a specific task.
    pub fn state_of(&self, task_id: &Uuid) -> Option<TaskState> {
        self.tasks.get(task_id).map(|t| t.state)
    }

    /// Iterate over all task timelines in first-seen order.
    pub fn timelines(&self) -> impl Iterator<Item = &TaskTimeline> {
        self.order.iter().filter_map(move |id| self.tasks.get(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{SourceLocation, Value};
    use chrono::Utc;

    fn start(task_id: Uuid, name: &str) -> ExecutionEvent {
        ExecutionEvent::AsyncTaskStart {
            id: Uuid::new_v4(),
            task_id,
            task_name: name.to_string(),
            spawned_from: SourceLocation::new("main.rs".to_string(), 1, 1),
            timestamp: Utc::now(),
        }
    }

    fn await_on(task_id: Uuid, on: &str) -> ExecutionEvent {
        ExecutionEvent::AsyncTaskAwait {
            id: Uuid::new_v4(),
            task_id,
            awaiting_on: on.to_string(),
            location: SourceLocation::new("worker.rs".to_string(), 2, 1),
            timestamp: Utc::now(),
        }
    }

    fn resume(task_id: Uuid, value: Option<Value>) -> ExecutionEvent {
        ExecutionEvent::AsyncTaskResume {
            id: Uuid::new_v4(),
            task_id,
            resumed_with: value,
            location: SourceLocation::new("worker.rs".to_string(), 3, 1),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_interleaved_tasks_are_separated() {
        let task_a = Uuid::new_v4();
        let task_b = Uuid::new_v4();

        // Interleave events from two tasks.
        let events = vec![
            start(task_a, "alpha"),
            start(task_b, "beta"),
            await_on(task_a, "db_query"),
            await_on(task_b, "http_get"),
            resume(task_b, Some(Value::String("ok".to_string()))),
            resume(task_a, Some(Value::Integer(7))),
            await_on(task_a, "db_query_2"),
        ];

        let tracker = AsyncTaskTracker::from_events(&events);

        assert_eq!(tracker.task_count(), 2);
        assert_eq!(tracker.task_ids(), &[task_a, task_b]);

        // Task A: start -> await -> resume -> await  (currently Awaiting)
        let a = tracker.timeline(&task_a).expect("task a");
        assert_eq!(a.task_name.as_deref(), Some("alpha"));
        assert_eq!(a.len(), 4);
        assert_eq!(a.state, TaskState::Awaiting);
        assert_eq!(a.events[0].event_type(), "async_task_start");
        assert_eq!(a.events[1].event_type(), "async_task_await");
        assert_eq!(a.events[2].event_type(), "async_task_resume");
        assert_eq!(a.events[3].event_type(), "async_task_await");

        // Task B: start -> await -> resume  (currently Running)
        let b = tracker.timeline(&task_b).expect("task b");
        assert_eq!(b.task_name.as_deref(), Some("beta"));
        assert_eq!(b.len(), 3);
        assert_eq!(b.state, TaskState::Running);
        assert_eq!(b.events[2].event_type(), "async_task_resume");

        assert_eq!(tracker.state_of(&task_a), Some(TaskState::Awaiting));
        assert_eq!(tracker.state_of(&task_b), Some(TaskState::Running));
    }

    #[test]
    fn test_mark_completed_and_unknown_task() {
        let task_a = Uuid::new_v4();
        let events = vec![start(task_a, "alpha"), resume(task_a, None)];
        let mut tracker = AsyncTaskTracker::from_events(&events);

        assert_eq!(tracker.state_of(&task_a), Some(TaskState::Running));
        tracker.mark_completed(task_a);
        assert_eq!(tracker.state_of(&task_a), Some(TaskState::Completed));

        assert_eq!(tracker.state_of(&Uuid::new_v4()), None);
    }

    #[test]
    fn test_non_async_events_ignored() {
        let events = vec![ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "f".to_string(),
            args: std::collections::HashMap::new(),
            location: SourceLocation::unknown(),
            timestamp: Utc::now(),
        }];
        let tracker = AsyncTaskTracker::from_events(&events);
        assert_eq!(tracker.task_count(), 0);
    }
}
