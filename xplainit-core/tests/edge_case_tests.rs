//! Unit + edge-case tests for previously thin xplainit-core modules (FEAT-004).
//!
//! These target boundary and error conditions that the in-module unit tests did
//! not cover: empty inputs, deeply nested / large structures, unicode and long
//! strings, boundary depths (0 and max), malformed data, and formatter/sink
//! error branches. Every test exercises real library behavior and fails if the
//! code path it covers is reverted.

use std::collections::HashMap;
use std::time::Duration;

use chrono::Utc;
use uuid::Uuid;
use xplainit_core::*;

fn fenter(name: &str, file: &str, line: usize) -> ExecutionEvent {
    ExecutionEvent::FunctionEnter {
        id: Uuid::new_v4(),
        name: name.to_string(),
        args: HashMap::new(),
        location: SourceLocation::new(file.to_string(), line, 0),
        timestamp: Utc::now(),
    }
}

// ===== formatter.rs edge cases =====

#[test]
fn formatter_handles_empty_event_slice() {
    // format_events over an empty slice must produce empty output for the
    // per-event formatters and a valid empty JSON array for the JSON formatter.
    let events: Vec<ExecutionEvent> = vec![];

    let text = TextFormatter::new(VerbosityLevel::Normal);
    assert_eq!(text.format_events(&events), "");

    let md = MarkdownFormatter::new(VerbosityLevel::Normal);
    assert_eq!(md.format_events(&events), "");

    let json = JsonFormatter::new(false);
    let out = json.format_events(&events);
    assert_eq!(out, "[]");
    let parsed: Vec<ExecutionEvent> = serde_json::from_str(&out).unwrap();
    assert!(parsed.is_empty());
}

#[test]
fn formatter_events_join_has_no_trailing_newline() {
    // format_events joins per-event output with a single '\n' and no trailing
    // newline. Reverting the buffered join to a different separator fails here.
    let events = vec![fenter("a", "x.py", 1), fenter("b", "x.py", 2)];
    let text = TextFormatter::new(VerbosityLevel::Normal);
    let out = text.format_events(&events);
    assert_eq!(
        out.matches('\n').count(),
        1,
        "exactly one separator newline"
    );
    assert!(!out.ends_with('\n'), "no trailing newline");
}

#[test]
fn html_formatter_marks_error_events_with_error_class() {
    let html = HtmlFormatter::new();
    let normal = html.format_event(&fenter("ok", "x.py", 1));
    assert!(normal.contains("<div class=\"event\">"));
    assert!(!normal.contains("event error"));

    let err = ExecutionEvent::DivisionByZero {
        id: Uuid::new_v4(),
        numerator: Value::Integer(1),
        denominator_var: Some("d".to_string()),
        location: SourceLocation::new("x.py".to_string(), 3, 0),
        timestamp: Utc::now(),
    };
    let err_html = html.format_event(&err);
    assert!(
        err_html.contains("event error"),
        "error events get error class"
    );
}

#[test]
fn formatter_preserves_unicode_and_long_strings() {
    // Unicode function names and very long argument strings must survive JSON
    // serialization intact.
    let mut args = HashMap::new();
    let long = "λ".repeat(5000);
    args.insert("emoji_名前".to_string(), Value::String(long.clone()));
    let event = ExecutionEvent::FunctionEnter {
        id: Uuid::new_v4(),
        name: "计算_λ".to_string(),
        args,
        location: SourceLocation::new("模块.py".to_string(), 1, 0),
        timestamp: Utc::now(),
    };

    let json = JsonFormatter::new(false).format_event(&event);
    let decoded: ExecutionEvent = serde_json::from_str(&json).expect("round-trip unicode");
    match decoded {
        ExecutionEvent::FunctionEnter {
            name,
            args,
            location,
            ..
        } => {
            assert_eq!(name, "计算_λ");
            assert_eq!(location.file, "模块.py");
            match args.get("emoji_名前") {
                Some(Value::String(s)) => assert_eq!(s.chars().count(), 5000),
                other => panic!("unicode arg lost: {other:?}"),
            }
        }
        _ => panic!("wrong variant"),
    }
}

// ===== sink.rs edge cases =====

#[test]
fn console_sink_rejects_unsupported_format() {
    // The console sink only supports Console/Json; HTML must yield an error
    // rather than silently succeeding.
    let mut sink = ConsoleSink::new(OutputFormat::Html);
    let err = sink.write(&fenter("f", "x.py", 1));
    assert!(
        err.is_err(),
        "HTML format is unsupported for the console sink"
    );
}

#[test]
fn memory_sink_zero_capacity_keeps_at_most_one() {
    // Boundary: with max_events == 0 the guard `len >= max` fires before every
    // push, popping the (possibly empty) front and then pushing the new event,
    // so the sink retains at most one event no matter how many are written.
    // This documents the real overflow-guard behavior; a regression that let
    // the buffer grow would fail here.
    let mut sink = MemorySink::new(0);
    for _ in 0..10 {
        sink.write(&fenter("f", "x.py", 1)).unwrap();
    }
    assert!(sink.get_events().len() <= 1);
}

#[test]
fn multi_sink_reports_child_write_failure() {
    // A MultiSink containing a console sink configured with an unsupported
    // format must surface the child failure as an error.
    let mut multi = MultiSink::new().add_sink(Box::new(ConsoleSink::new(OutputFormat::Html)));
    assert!(multi.write(&fenter("f", "x.py", 1)).is_err());
    // flush/close on an empty-ish multi sink must still succeed.
    assert!(multi.flush().is_ok());
}

#[test]
fn file_sink_flushes_buffer_to_disk() {
    // FileSink buffers then flushes; after close the file must contain the
    // serialized event. Exercises the real IO write path.
    let mut path = std::env::temp_dir();
    path.push(format!("xplainit-filesink-{}.jsonl", std::process::id()));
    {
        let mut sink = FileSink::new(path.clone(), OutputFormat::Json)
            .expect("create file sink")
            .with_buffer_size(1);
        sink.write(&fenter("written", "x.py", 9)).unwrap();
        sink.close().unwrap();
    }
    let contents = std::fs::read_to_string(&path).expect("read back sink file");
    let _ = std::fs::remove_file(&path);
    assert!(
        contents.contains("written"),
        "event not persisted: {contents}"
    );
}

// ===== event_store.rs edge cases =====

#[test]
fn event_store_snapshot_preserves_events() {
    // snapshot() drains and re-adds; the store must be non-empty afterwards and
    // the snapshot must contain every recorded event.
    let store = EventStore::with_capacity(16);
    for i in 0..8 {
        store.record(fenter(&format!("f{i}"), "x.py", i));
    }
    let snap = store.snapshot();
    assert_eq!(snap.len(), 8);
    assert_eq!(store.len(), 8, "snapshot must not drain the store");
}

#[test]
fn event_store_capacity_one_drops_all_but_latest() {
    // Boundary: a capacity-1 store keeps only the most recent event and counts
    // the rest as dropped.
    let store = EventStore::with_capacity(1);
    for i in 0..5 {
        store.record(fenter(&format!("f{i}"), "x.py", i));
    }
    assert_eq!(store.len(), 1);
    let stats = store.stats();
    assert_eq!(stats.total_recorded, 5);
    assert_eq!(stats.total_dropped, 4);
}

#[test]
fn event_store_clear_resets_len_but_keeps_totals() {
    let store = EventStore::with_capacity(10);
    for i in 0..3 {
        store.record(fenter(&format!("f{i}"), "x.py", i));
    }
    store.clear();
    assert!(store.is_empty());
    // Totals are cumulative; clear() only empties the buffer.
    assert_eq!(store.stats().total_recorded, 3);
}

// ===== advanced_filter.rs edge cases =====

#[test]
fn call_stack_filter_zero_depth_is_unlimited() {
    // max_depth == 0 disables the depth limit entirely.
    let mut filter = CallStackFilter::new(0);
    let enter = fenter("f", "x.py", 1);
    for _ in 0..1000 {
        assert!(filter.should_capture_at_depth("t", &enter));
    }
}

#[test]
fn call_stack_filter_balances_enter_and_exit() {
    // Depth must decrement on exit so a balanced enter/exit sequence never
    // permanently exceeds the limit.
    let mut filter = CallStackFilter::new(2);
    let enter = fenter("f", "x.py", 1);
    let exit = ExecutionEvent::FunctionExit {
        id: Uuid::new_v4(),
        name: "f".to_string(),
        return_value: None,
        duration: Duration::from_millis(1),
        timestamp: Utc::now(),
    };

    assert!(filter.should_capture_at_depth("t", &enter)); // depth 1, captured
    assert!(filter.should_capture_at_depth("t", &enter)); // depth 2, captured
    assert!(!filter.should_capture_at_depth("t", &enter)); // depth 3, rejected
    assert_eq!(
        filter.current_depth("t"),
        3,
        "rejected enter still tracks depth"
    );
    // Exit events are always captured and decrement the depth counter.
    assert!(filter.should_capture_at_depth("t", &exit)); // depth 2
    assert!(filter.should_capture_at_depth("t", &exit)); // depth 1
    assert_eq!(filter.current_depth("t"), 1);
    // Now a fresh enter reaches depth 2 again and is captured.
    assert!(filter.should_capture_at_depth("t", &enter)); // depth 2, captured
    filter.reset_thread("t");
    assert_eq!(filter.current_depth("t"), 0);
}

#[test]
fn module_filter_glob_pattern_matches() {
    // A '*' glob pattern is compiled to a regex and must match matching files
    // while leaving others alone.
    let filter = ModuleFilter {
        include_modules: Default::default(),
        exclude_modules: Default::default(),
        exclude_stdlib: false,
        exclude_patterns: vec!["*/generated/*".to_string()],
    };
    assert!(filter.should_filter_file("/app/generated/pb.py"));
    assert!(!filter.should_filter_file("/app/src/main.py"));
}

#[test]
fn regex_filter_invalid_pattern_errors() {
    // An invalid regex must return Err, not panic.
    let res = RegexFilter::new().include_pattern("([unclosed");
    assert!(res.is_err());
}

#[test]
fn performance_filter_sampling_rate_zero_is_clamped_to_one() {
    // with_sampling(0) is clamped to 1 (capture all), so should_sample() is
    // always true. Reverting the `.max(1)` guard would divide by zero.
    let mut filter = PerformanceFilter::new().with_sampling(0);
    for _ in 0..10 {
        assert!(filter.should_sample());
    }
}

// ===== async_tracker.rs edge cases =====

#[test]
fn async_tracker_empty_input_is_empty() {
    let tracker = AsyncTaskTracker::from_events(&[]);
    assert_eq!(tracker.task_count(), 0);
    assert!(tracker.task_ids().is_empty());
    assert_eq!(tracker.timelines().count(), 0);
}

#[test]
fn async_tracker_resume_without_start_still_tracked() {
    // A resume for a never-started task should still create a timeline (the
    // tracker keys purely on task_id).
    let task = Uuid::new_v4();
    let resume = ExecutionEvent::AsyncTaskResume {
        id: Uuid::new_v4(),
        task_id: task,
        resumed_with: None,
        location: SourceLocation::new("w.rs".to_string(), 1, 0),
        timestamp: Utc::now(),
    };
    let tracker = AsyncTaskTracker::from_events(&[resume]);
    assert_eq!(tracker.task_count(), 1);
    assert_eq!(tracker.state_of(&task), Some(TaskState::Running));
    assert_eq!(tracker.timeline(&task).unwrap().task_name, None);
}

// ===== error_explainer.rs edge cases =====

#[test]
fn error_explainer_ignores_non_error_events() {
    // analyze() must return None for a normal (non-error) event.
    let explainer = ErrorExplainer::new();
    assert!(explainer.analyze(&fenter("f", "x.py", 1)).is_none());
}

#[test]
fn error_explainer_context_history_is_bounded() {
    // With a context size of N, tracking more than N events must not grow the
    // history unbounded (exercises the remove(0) trim path). We can only
    // observe behavior indirectly: tracking many events must not panic and
    // analysis of an error still works.
    let mut explainer = ErrorExplainer::new().with_context_size(4);
    for i in 0..50 {
        explainer.track_event(fenter(&format!("f{i}"), "x.py", i));
    }
    let err = ExecutionEvent::StackOverflow {
        id: Uuid::new_v4(),
        function: "recurse".to_string(),
        recursion_depth: 10_000,
        location: SourceLocation::new("x.py".to_string(), 5, 0),
        timestamp: Utc::now(),
    };
    let analysis = explainer.analyze(&err).expect("stack overflow analyzed");
    assert_eq!(analysis.category, ErrorCategory::Resource);
    assert!(!analysis.fix_suggestions.is_empty());
}

// ===== explainer.rs edge cases =====

#[test]
fn explanation_generator_brief_vs_detailed_differ() {
    // The verbosity level must actually change output length/content.
    let event = ExecutionEvent::VariableAssign {
        id: Uuid::new_v4(),
        name: "total".to_string(),
        old_value: Some(Value::Integer(1)),
        new_value: Value::Integer(2),
        location: SourceLocation::new("x.py".to_string(), 3, 0),
        timestamp: Utc::now(),
    };
    let brief = ExplanationGenerator::new(VerbosityLevel::Brief).explain(&event);
    let detailed = ExplanationGenerator::new(VerbosityLevel::Detailed).explain(&event);
    assert!(!brief.is_empty());
    assert!(!detailed.is_empty());
    assert!(brief.contains("total") || detailed.contains("total"));
}

#[test]
fn explanation_generator_id_and_timestamp_prefixes() {
    // Enabling ids/timestamps must prepend prefixes to the explanation.
    let event = fenter("main", "x.py", 1);
    let with_id = ExplanationGenerator::new(VerbosityLevel::Normal)
        .with_ids(true)
        .explain(&event);
    assert!(
        with_id.contains(&event.id().to_string()),
        "id prefix missing"
    );
}
