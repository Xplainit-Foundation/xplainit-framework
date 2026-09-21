//! Derived views over a trace: error aggregation, call graph, and timeline.
//!
//! All views reuse `xplainit_core` types. Error analysis is delegated to
//! `xplainit_core::ErrorExplainer` so the dashboard does not reimplement any
//! error reasoning.

use serde_json::{json, Value as JsonValue};
use std::collections::BTreeMap;
use xplainit_core::{AsyncTaskTracker, ErrorExplainer, ExecutionEvent};

/// Map an error event to a stable, human readable error type label used for
/// aggregation counts. Non error events return `None`.
pub fn error_type_label(event: &ExecutionEvent) -> Option<String> {
    match event {
        ExecutionEvent::Exception { error_type, .. } => Some(error_type.clone()),
        ExecutionEvent::RuntimeError { error_type, .. } => Some(error_type.clone()),
        ExecutionEvent::SyntaxError { .. } => Some("SyntaxError".to_string()),
        ExecutionEvent::TypeError { .. } => Some("TypeError".to_string()),
        ExecutionEvent::NullPointerError { .. } => Some("NullPointerError".to_string()),
        ExecutionEvent::IndexOutOfBounds { .. } => Some("IndexOutOfBounds".to_string()),
        ExecutionEvent::DivisionByZero { .. } => Some("DivisionByZero".to_string()),
        ExecutionEvent::StackOverflow { .. } => Some("StackOverflow".to_string()),
        ExecutionEvent::Panic { .. } => Some("Panic".to_string()),
        ExecutionEvent::InfiniteLoopDetected { .. } => Some("InfiniteLoopDetected".to_string()),
        ExecutionEvent::DeadlockDetected { .. } => Some("DeadlockDetected".to_string()),
        ExecutionEvent::MemoryLeakDetected { .. } => Some("MemoryLeakDetected".to_string()),
        _ => None,
    }
}

/// Build the error aggregation summary as a JSON value.
///
/// Includes:
/// - total event count and error count,
/// - counts grouped by error type label,
/// - per error analyses produced by `ErrorExplainer`.
pub fn build_summary(events: &[ExecutionEvent]) -> JsonValue {
    let mut explainer = ErrorExplainer::new();
    for event in events {
        explainer.track_event(event.clone());
    }

    let mut counts_by_type: BTreeMap<String, usize> = BTreeMap::new();
    let mut analyses: Vec<JsonValue> = Vec::new();

    for event in events.iter().filter(|e| e.is_error()) {
        if let Some(label) = error_type_label(event) {
            *counts_by_type.entry(label).or_insert(0) += 1;
        }

        if let Some(analysis) = explainer.analyze(event) {
            let location = analysis.event.location();
            analyses.push(json!({
                "event_type": analysis.event.event_type(),
                "error_type": error_type_label(&analysis.event),
                "severity": format!("{:?}", analysis.severity),
                "category": format!("{:?}", analysis.category),
                "root_cause": analysis.root_cause,
                "location": {
                    "file": location.file,
                    "line": location.line,
                    "column": location.column,
                },
                "fix_suggestions": analysis.fix_suggestions,
                "prevention_tips": analysis.prevention_tips,
                "related_errors": analysis.related_errors,
            }));
        }
    }

    // Derive the error count from the authoritative `is_error()` predicate
    // rather than the labelled counts. `error_type_label` is only used for the
    // grouped `counts_by_type` breakdown; if a future error variant is added to
    // `ExecutionEvent::is_error()` without a matching `error_type_label` arm it
    // would be missing from `counts_by_type`, but the total `error_count` must
    // still reflect every event that `is_error()` reports.
    let error_count = events.iter().filter(|e| e.is_error()).count();
    debug_assert!(
        counts_by_type.values().sum::<usize>() <= error_count,
        "labelled error counts must never exceed the is_error() total"
    );

    json!({
        "total_events": events.len(),
        "error_count": error_count,
        "counts_by_type": counts_by_type,
        "analyses": analyses,
    })
}

/// Build a per async-task view from the trace using `AsyncTaskTracker`.
///
/// Groups the async lifecycle events (`AsyncTaskStart`/`AsyncTaskAwait`/
/// `AsyncTaskResume`) by `task_id`, reconstructing each task's ordered timeline
/// and current state. This surfaces the per-task grouping that the tracker
/// computes, which is otherwise invisible in the flat timeline and call graph.
pub fn build_task_view(events: &[ExecutionEvent]) -> JsonValue {
    let tracker = AsyncTaskTracker::from_events(events);

    let tasks: Vec<JsonValue> = tracker
        .timelines()
        .map(|timeline| {
            let events: Vec<JsonValue> = timeline
                .events
                .iter()
                .map(|event| {
                    let location = event.location();
                    json!({
                        "event_type": event.event_type(),
                        "timestamp": event.timestamp().to_rfc3339(),
                        "location": {
                            "file": location.file,
                            "line": location.line,
                        },
                    })
                })
                .collect();

            json!({
                "task_id": timeline.task_id.to_string(),
                "task_name": timeline.task_name,
                "state": format!("{:?}", timeline.state),
                "event_count": timeline.len(),
                "events": events,
            })
        })
        .collect();

    json!({
        "task_count": tracker.task_count(),
        "tasks": tasks,
    })
}

/// A single node in the derived call graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallNode {
    /// Function name.
    pub name: String,
    /// Zero based nesting depth at the point of entry.
    pub depth: usize,
    /// Index of the parent node in the flattened node list, if any.
    pub parent: Option<usize>,
    /// Indices of direct children in the flattened node list.
    pub children: Vec<usize>,
}

/// Build a call graph from `FunctionEnter` and `FunctionExit` events.
///
/// Nesting is derived from the enter/exit ordering: entering a function while
/// another is active makes the new function a child of the active one. Exits
/// pop the current stack. Unmatched exits are ignored.
pub fn build_call_graph(events: &[ExecutionEvent]) -> Vec<CallNode> {
    let mut nodes: Vec<CallNode> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();

    for event in events {
        match event {
            ExecutionEvent::FunctionEnter { name, .. } => {
                let parent = stack.last().copied();
                let index = nodes.len();
                nodes.push(CallNode {
                    name: name.clone(),
                    depth: stack.len(),
                    parent,
                    children: Vec::new(),
                });
                if let Some(parent_index) = parent {
                    nodes[parent_index].children.push(index);
                }
                stack.push(index);
            }
            ExecutionEvent::FunctionExit { .. } => {
                stack.pop();
            }
            _ => {}
        }
    }

    nodes
}

/// Serialize a call graph to JSON (flattened node list with parent/children).
pub fn call_graph_json(nodes: &[CallNode]) -> JsonValue {
    let items: Vec<JsonValue> = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            json!({
                "index": index,
                "name": node.name,
                "depth": node.depth,
                "parent": node.parent,
                "children": node.children,
            })
        })
        .collect();

    json!({ "nodes": items })
}

/// Build a timeline from event timestamps.
///
/// Each entry captures the ordinal position, event type, source location,
/// error flag and the RFC3339 timestamp so the frontend can lay events out on
/// a time axis.
pub fn build_timeline(events: &[ExecutionEvent]) -> JsonValue {
    let items: Vec<JsonValue> = events
        .iter()
        .enumerate()
        .map(|(index, event)| {
            let location = event.location();
            json!({
                "index": index,
                "event_type": event.event_type(),
                "timestamp": event.timestamp().to_rfc3339(),
                "is_error": event.is_error(),
                "location": {
                    "file": location.file,
                    "line": location.line,
                },
            })
        })
        .collect();

    json!({ "events": items })
}

/// Serialize one event as an SSE data payload line body (the JSON only, without
/// the `data: ` prefix or trailing blank line).
pub fn event_stream_payload(index: usize, event: &ExecutionEvent) -> JsonValue {
    json!({
        "index": index,
        "event_type": event.event_type(),
        "timestamp": event.timestamp().to_rfc3339(),
        "is_error": event.is_error(),
        "event": event,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(json: &str) -> ExecutionEvent {
        serde_json::from_str(json).expect("valid fixture event")
    }

    fn enter(id_suffix: &str, name: &str) -> ExecutionEvent {
        event(&format!(
            r#"{{"FunctionEnter": {{
                "id": "550e8400-e29b-41d4-a716-4466554400{id_suffix}",
                "name": "{name}",
                "args": {{}},
                "location": {{"file": "app.py", "line": 1, "column": 1, "offset": 0}},
                "timestamp": "2024-01-01T00:00:00Z"
            }}}}"#
        ))
    }

    fn exit(id_suffix: &str, name: &str) -> ExecutionEvent {
        event(&format!(
            r#"{{"FunctionExit": {{
                "id": "550e8400-e29b-41d4-a716-4466554400{id_suffix}",
                "name": "{name}",
                "return_value": null,
                "duration": {{"secs": 0, "nanos": 0}},
                "timestamp": "2024-01-01T00:00:02Z"
            }}}}"#
        ))
    }

    fn div_zero(id_suffix: &str) -> ExecutionEvent {
        event(&format!(
            r#"{{"DivisionByZero": {{
                "id": "550e8400-e29b-41d4-a716-4466554400{id_suffix}",
                "numerator": {{"Integer": 10}},
                "denominator_var": "x",
                "location": {{"file": "app.py", "line": 12, "column": 5, "offset": 0}},
                "timestamp": "2024-01-01T00:00:01Z"
            }}}}"#
        ))
    }

    fn type_error(id_suffix: &str) -> ExecutionEvent {
        event(&format!(
            r#"{{"TypeError": {{
                "id": "550e8400-e29b-41d4-a716-4466554400{id_suffix}",
                "expected": "integer",
                "got": "string",
                "value": {{"String": "hi"}},
                "operation": "add",
                "location": {{"file": "app.py", "line": 20, "column": 3, "offset": 0}},
                "timestamp": "2024-01-01T00:00:03Z"
            }}}}"#
        ))
    }

    fn async_start(id_suffix: &str, task_id: &str, name: &str) -> ExecutionEvent {
        event(&format!(
            r#"{{"AsyncTaskStart": {{
                "id": "550e8400-e29b-41d4-a716-4466554400{id_suffix}",
                "task_id": "{task_id}",
                "task_name": "{name}",
                "spawned_from": {{"file": "main.rs", "line": 1, "column": 1, "offset": 0}},
                "timestamp": "2024-01-01T00:00:00Z"
            }}}}"#
        ))
    }

    fn async_await(id_suffix: &str, task_id: &str, on: &str) -> ExecutionEvent {
        event(&format!(
            r#"{{"AsyncTaskAwait": {{
                "id": "550e8400-e29b-41d4-a716-4466554400{id_suffix}",
                "task_id": "{task_id}",
                "awaiting_on": "{on}",
                "location": {{"file": "worker.rs", "line": 2, "column": 1, "offset": 0}},
                "timestamp": "2024-01-01T00:00:01Z"
            }}}}"#
        ))
    }

    #[test]
    fn aggregates_error_counts() {
        let events = vec![
            enter("10", "main"),
            div_zero("11"),
            type_error("12"),
            div_zero("13"),
        ];
        let summary = build_summary(&events);

        assert_eq!(summary["total_events"], 4);
        assert_eq!(summary["error_count"], 3);
        assert_eq!(summary["counts_by_type"]["DivisionByZero"], 2);
        assert_eq!(summary["counts_by_type"]["TypeError"], 1);
        assert_eq!(summary["analyses"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn summary_has_no_errors_for_clean_trace() {
        let events = vec![enter("10", "main"), exit("10", "main")];
        let summary = build_summary(&events);
        assert_eq!(summary["error_count"], 0);
        assert!(summary["counts_by_type"].as_object().unwrap().is_empty());
    }

    #[test]
    fn builds_nested_call_graph() {
        // main -> helper -> leaf, then unwind.
        let events = vec![
            enter("10", "main"),
            enter("11", "helper"),
            enter("12", "leaf"),
            exit("12", "leaf"),
            exit("11", "helper"),
            exit("10", "main"),
        ];
        let nodes = build_call_graph(&events);

        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0].name, "main");
        assert_eq!(nodes[0].depth, 0);
        assert_eq!(nodes[0].parent, None);
        assert_eq!(nodes[0].children, vec![1]);

        assert_eq!(nodes[1].name, "helper");
        assert_eq!(nodes[1].depth, 1);
        assert_eq!(nodes[1].parent, Some(0));
        assert_eq!(nodes[1].children, vec![2]);

        assert_eq!(nodes[2].name, "leaf");
        assert_eq!(nodes[2].depth, 2);
        assert_eq!(nodes[2].parent, Some(1));
        assert!(nodes[2].children.is_empty());
    }

    #[test]
    fn call_graph_handles_siblings() {
        // main -> a (exit) -> b (exit)
        let events = vec![
            enter("10", "main"),
            enter("11", "a"),
            exit("11", "a"),
            enter("12", "b"),
            exit("12", "b"),
            exit("10", "main"),
        ];
        let nodes = build_call_graph(&events);
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0].children, vec![1, 2]);
        assert_eq!(nodes[1].parent, Some(0));
        assert_eq!(nodes[2].parent, Some(0));
    }

    #[test]
    fn call_graph_json_round_trips() {
        let events = vec![enter("10", "main"), enter("11", "helper")];
        let nodes = build_call_graph(&events);
        let value = call_graph_json(&nodes);
        let arr = value["nodes"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], "main");
        assert_eq!(arr[1]["parent"], 0);
    }

    #[test]
    fn timeline_preserves_order_and_flags() {
        let events = vec![enter("10", "main"), div_zero("11")];
        let timeline = build_timeline(&events);
        let arr = timeline["events"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["is_error"], false);
        assert_eq!(arr[1]["is_error"], true);
        assert_eq!(arr[1]["event_type"], "division_by_zero");
    }

    #[test]
    fn error_count_tracks_is_error_not_labels() {
        // Every error event here has a label arm, so the count matches, but the
        // count is derived from is_error() so it stays correct even for a future
        // error variant without an error_type_label arm.
        let events = vec![enter("10", "main"), div_zero("11"), type_error("12")];

        // Sanity: all error events are counted by is_error().
        let expected = events.iter().filter(|e| e.is_error()).count();
        assert_eq!(expected, 2);

        let summary = build_summary(&events);
        assert_eq!(summary["error_count"], 2);
        // error_count must equal the is_error() total, independent of the
        // labelled breakdown in counts_by_type.
        let labelled: u64 = summary["counts_by_type"]
            .as_object()
            .unwrap()
            .values()
            .map(|v| v.as_u64().unwrap())
            .sum();
        assert_eq!(summary["error_count"].as_u64().unwrap(), labelled);
    }

    #[test]
    fn task_view_groups_events_by_task() {
        let task_a = "11111111-1111-4111-8111-111111111111";
        let task_b = "22222222-2222-4222-8222-222222222222";
        let events = vec![
            async_start("10", task_a, "alpha"),
            async_start("11", task_b, "beta"),
            async_await("12", task_a, "db_query"),
        ];

        let view = build_task_view(&events);
        assert_eq!(view["task_count"], 2);

        let tasks = view["tasks"].as_array().unwrap();
        assert_eq!(tasks.len(), 2);

        // First-seen order is preserved: task_a then task_b.
        assert_eq!(tasks[0]["task_id"], task_a);
        assert_eq!(tasks[0]["task_name"], "alpha");
        assert_eq!(tasks[0]["state"], "Awaiting");
        assert_eq!(tasks[0]["event_count"], 2);

        assert_eq!(tasks[1]["task_id"], task_b);
        assert_eq!(tasks[1]["task_name"], "beta");
        assert_eq!(tasks[1]["state"], "Running");
        assert_eq!(tasks[1]["event_count"], 1);
    }

    #[test]
    fn task_view_is_empty_without_async_events() {
        let events = vec![enter("10", "main"), exit("10", "main")];
        let view = build_task_view(&events);
        assert_eq!(view["task_count"], 0);
        assert!(view["tasks"].as_array().unwrap().is_empty());
    }

    #[test]
    fn stream_payload_is_serializable() {
        let events = [enter("10", "main")];
        let payload = event_stream_payload(0, &events[0]);
        let serialized = serde_json::to_string(&payload).expect("serialize");
        assert!(serialized.contains("function_enter"));
        assert!(serialized.contains("\"index\":0"));
        // The embedded event keeps the externally tagged core shape.
        assert!(serialized.contains("FunctionEnter"));
    }
}
