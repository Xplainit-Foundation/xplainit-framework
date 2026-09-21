//! Revert-sensitive tests for the FEAT-003 performance optimizations.
//!
//! These assert the *observable* behavior that must stay identical after the
//! allocation/clone reductions:
//!   * serialized JSON output is byte-for-byte unchanged (golden test),
//!   * `location_ref()` agrees with the owned `location()` for every variant,
//!   * `MemorySink` keeps FIFO ordering when it overflows.
//!
//! Each test fails if the corresponding optimization is reverted in a way that
//! changes semantics.

use std::collections::HashMap;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use uuid::Uuid;
use xplainit_core::{
    Config, EventFilter, ExecutionEvent, JsonFormatter, Language, MemorySink, OutputFormatter,
    PathFilter, SourceLocation, StackFrame, Value,
};

/// Build a fixed set of events with deterministic ids/timestamps so their
/// serialization is stable across runs. Covers the variants touched by the
/// refactor (with and without a location) plus nested Value payloads.
fn golden_events() -> Vec<ExecutionEvent> {
    let ts = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
    let id = |n: u128| Uuid::from_u128(n);
    let loc = SourceLocation {
        file: "app/module.py".to_string(),
        line: 42,
        column: 7,
        offset: 100,
    };

    // Use a single-entry map so the JSON object ordering is deterministic
    // (multi-key HashMap iteration order is randomized per process, which
    // would make a byte-for-byte golden assertion flaky rather than
    // meaningful). A nested Array still exercises nested Value serialization.
    let mut args = HashMap::new();
    args.insert(
        "payload".to_string(),
        Value::Array(vec![Value::Bool(true), Value::Null, Value::Integer(3)]),
    );

    vec![
        ExecutionEvent::FunctionEnter {
            id: id(1),
            name: "compute".to_string(),
            args,
            location: loc.clone(),
            timestamp: ts,
        },
        // No location variant (exercises the `None` arm of location_ref).
        ExecutionEvent::FunctionExit {
            id: id(2),
            name: "compute".to_string(),
            return_value: Some(Value::Float(2.5)),
            duration: Duration::from_millis(12),
            timestamp: ts,
        },
        ExecutionEvent::VariableAssign {
            id: id(3),
            name: "total".to_string(),
            old_value: Some(Value::Integer(0)),
            new_value: Value::Integer(9),
            location: loc.clone(),
            timestamp: ts,
        },
        ExecutionEvent::Exception {
            id: id(4),
            error_type: "ValueError".to_string(),
            message: "bad input".to_string(),
            location: loc.clone(),
            stack_trace: vec![StackFrame {
                function_name: "compute".to_string(),
                location: loc.clone(),
                arguments: HashMap::new(),
            }],
            caught: false,
            timestamp: ts,
        },
        ExecutionEvent::AsyncTaskStart {
            id: id(5),
            task_id: id(99),
            task_name: "worker".to_string(),
            spawned_from: loc.clone(),
            timestamp: ts,
        },
    ]
}

/// GOLDEN: the compact JSON serialization of the canonical events must match
/// this exact string. If an optimization accidentally changes field values,
/// ordering, or shape, this fails.
#[test]
fn serialized_output_is_byte_for_byte_stable() {
    let events = golden_events();
    let formatter = JsonFormatter::new(false);
    let actual = formatter.format_events(&events);

    // Every map in `golden_events()` has at most one key, so the compact JSON
    // is fully deterministic and can be pinned byte-for-byte. If any
    // optimization changes field values, ordering, or the `"\n"`-free array
    // shape produced by `format_events`, this exact comparison fails.
    let expected = concat!(
        "[",
        "{\"FunctionEnter\":{",
        "\"id\":\"00000000-0000-0000-0000-000000000001\",",
        "\"name\":\"compute\",",
        "\"args\":{\"payload\":{\"Array\":[{\"Bool\":true},\"Null\",{\"Integer\":3}]}},",
        "\"location\":{\"file\":\"app/module.py\",\"line\":42,\"column\":7,\"offset\":100},",
        "\"timestamp\":\"2023-11-14T22:13:20Z\"}},",
        "{\"FunctionExit\":{",
        "\"id\":\"00000000-0000-0000-0000-000000000002\",",
        "\"name\":\"compute\",",
        "\"return_value\":{\"Float\":2.5},",
        "\"duration\":{\"secs\":0,\"nanos\":12000000},",
        "\"timestamp\":\"2023-11-14T22:13:20Z\"}},",
        "{\"VariableAssign\":{",
        "\"id\":\"00000000-0000-0000-0000-000000000003\",",
        "\"name\":\"total\",",
        "\"old_value\":{\"Integer\":0},",
        "\"new_value\":{\"Integer\":9},",
        "\"location\":{\"file\":\"app/module.py\",\"line\":42,\"column\":7,\"offset\":100},",
        "\"timestamp\":\"2023-11-14T22:13:20Z\"}},",
        "{\"Exception\":{",
        "\"id\":\"00000000-0000-0000-0000-000000000004\",",
        "\"error_type\":\"ValueError\",",
        "\"message\":\"bad input\",",
        "\"location\":{\"file\":\"app/module.py\",\"line\":42,\"column\":7,\"offset\":100},",
        "\"stack_trace\":[{\"function_name\":\"compute\",",
        "\"location\":{\"file\":\"app/module.py\",\"line\":42,\"column\":7,\"offset\":100},",
        "\"arguments\":{}}],",
        "\"caught\":false,",
        "\"timestamp\":\"2023-11-14T22:13:20Z\"}},",
        "{\"AsyncTaskStart\":{",
        "\"id\":\"00000000-0000-0000-0000-000000000005\",",
        "\"task_id\":\"00000000-0000-0000-0000-000000000063\",",
        "\"task_name\":\"worker\",",
        "\"spawned_from\":{\"file\":\"app/module.py\",\"line\":42,\"column\":7,\"offset\":100},",
        "\"timestamp\":\"2023-11-14T22:13:20Z\"}}",
        "]"
    );
    assert_eq!(actual, expected, "serialized golden output changed");

    // Round-trip must also be byte-stable (deterministic because all maps
    // have <= 1 key).
    let decoded: Vec<ExecutionEvent> =
        serde_json::from_str(&actual).expect("golden output must deserialize");
    let reserialized = formatter.format_events(&decoded);
    assert_eq!(
        actual, reserialized,
        "serialization is not stable across a round trip"
    );
}

/// GOLDEN (single-event): FunctionExit has no HashMap, so its compact JSON is
/// fully deterministic and can be pinned exactly.
#[test]
fn function_exit_json_is_exact() {
    let ts = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
    let event = ExecutionEvent::FunctionExit {
        id: Uuid::from_u128(2),
        name: "compute".to_string(),
        return_value: Some(Value::Float(2.5)),
        duration: Duration::from_millis(12),
        timestamp: ts,
    };
    let json = serde_json::to_string(&event).unwrap();
    let expected = "{\"FunctionExit\":{\"id\":\"00000000-0000-0000-0000-000000000002\",\
\"name\":\"compute\",\"return_value\":{\"Float\":2.5},\
\"duration\":{\"secs\":0,\"nanos\":12000000},\
\"timestamp\":\"2023-11-14T22:13:20Z\"}}";
    assert_eq!(json, expected);
}

/// `location_ref()` (the allocation-free borrow used on the filter hot path)
/// must agree with the owned `location()` for every variant. If the borrow
/// refactor drops or mismaps a variant, this fails.
#[test]
fn location_ref_matches_owned_location() {
    for event in golden_events() {
        match event.location_ref() {
            Some(loc) => assert_eq!(
                *loc,
                event.location(),
                "location_ref disagreed with location() for {}",
                event.event_type()
            ),
            None => assert_eq!(
                event.location(),
                SourceLocation::unknown(),
                "no-location variant {} should fall back to unknown()",
                event.event_type()
            ),
        }
    }
}

/// The `PathFilter` borrow refactor must keep the exact same accept/reject
/// decisions as before, including the `<unknown>` fallback for events that
/// carry no location.
#[test]
fn path_filter_decisions_unchanged() {
    let config = Config::new(Language::Python);
    let ts = Utc::now();

    let with_loc = ExecutionEvent::FunctionEnter {
        id: Uuid::new_v4(),
        name: "f".to_string(),
        args: HashMap::new(),
        location: SourceLocation::new("src/app.py".to_string(), 1, 0),
        timestamp: ts,
    };
    // FunctionExit carries no location -> path is "<unknown>".
    let no_loc = ExecutionEvent::FunctionExit {
        id: Uuid::new_v4(),
        name: "f".to_string(),
        return_value: None,
        duration: Duration::from_millis(1),
        timestamp: ts,
    };

    // Exclude by real path.
    let excl = PathFilter::new().exclude("app.py");
    assert!(!excl.should_capture(&with_loc, &config));
    assert!(excl.should_capture(&no_loc, &config));

    // Include list matching the "<unknown>" sentinel must still capture the
    // location-less event (proving the fallback string is preserved).
    let incl_unknown = PathFilter::new().include("<unknown>");
    assert!(incl_unknown.should_capture(&no_loc, &config));
    assert!(!incl_unknown.should_capture(&with_loc, &config));
}

/// `MemorySink` must evict the OLDEST event on overflow (FIFO). The VecDeque
/// refactor must preserve this ordering; a LIFO/other regression fails here.
#[test]
fn memory_sink_overflow_is_fifo() {
    use xplainit_core::EventSink;
    let mut sink = MemorySink::new(3);
    let ts = Utc::now();
    for i in 0..5u128 {
        let event = ExecutionEvent::VariableDeclaration {
            id: Uuid::from_u128(i),
            name: format!("v{i}"),
            value: Some(Value::Integer(i as i64)),
            var_type: None,
            is_const: false,
            location: SourceLocation::new("t.py".to_string(), i as usize, 0),
            timestamp: ts,
        };
        sink.write(&event).unwrap();
    }

    let events = sink.get_events();
    assert_eq!(events.len(), 3, "capacity must be enforced");

    // The two oldest (v0, v1) were evicted; remaining are v2, v3, v4 in order.
    let names: Vec<String> = events
        .iter()
        .map(|e| match e {
            ExecutionEvent::VariableDeclaration { name, .. } => name.clone(),
            _ => panic!("unexpected event"),
        })
        .collect();
    assert_eq!(
        names,
        vec!["v2".to_string(), "v3".to_string(), "v4".to_string()]
    );
}
