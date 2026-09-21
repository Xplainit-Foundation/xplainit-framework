//! In-process load / stress tests for xplainit-core (FEAT-004).
//!
//! HONEST SCOPE NOTE: the production plan aspires to "1,000,000 events/sec,
//! 10,000 concurrent tasks, multi-hour soak". That literal target is NOT run
//! here: this is an offline CI sandbox with no dedicated load-testing rig, and
//! a multi-hour soak is not appropriate for a unit-test gate. Instead these
//! tests drive a *scaled-down but still substantial* volume (hundreds of
//! thousands of events) through the real pipeline and event store, single- and
//! multi-threaded, and assert:
//!   * completion without error/panic,
//!   * the event store's bounded buffer never exceeds capacity (no leak),
//!   * accounting invariants (recorded == kept + dropped) hold, and
//!   * throughput stays finite and stable within a generous time budget.
//!
//! The heaviest test is gated behind `#[ignore]` so the default `cargo test`
//! run stays fast; run it explicitly with `cargo test -- --ignored`.

use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use uuid::Uuid;
use xplainit_core::*;

fn make_event(i: usize) -> ExecutionEvent {
    // Mix normal and error events so error accounting is also exercised.
    if i.is_multiple_of(1000) {
        ExecutionEvent::DivisionByZero {
            id: Uuid::new_v4(),
            numerator: Value::Integer(i as i64),
            denominator_var: Some("d".to_string()),
            location: SourceLocation::new("load.py".to_string(), i % 500, 0),
            timestamp: Utc::now(),
        }
    } else {
        ExecutionEvent::VariableAssign {
            id: Uuid::new_v4(),
            name: format!("v{}", i % 64),
            old_value: None,
            new_value: Value::Integer(i as i64),
            location: SourceLocation::new("load.py".to_string(), i % 500, 0),
            timestamp: Utc::now(),
        }
    }
}

#[test]
fn load_pipeline_handles_200k_events_without_error() {
    // Scaled-down load test (see module note). 200k events through the full
    // filter -> processor -> sink pipeline must complete error-free and within
    // a generous budget.
    let n = 200_000usize;
    let filter = Box::new(AcceptAllFilter);
    let processors = ProcessorPipeline::new().add_processor(Box::new(PassThroughProcessor));
    // Bounded memory sink so this cannot grow without limit.
    let sink = Box::new(MemorySink::new(10_000));
    let mut pipeline = EventPipeline::new(filter, processors).add_sink(sink);
    let config = Config::new(Language::Python);

    let start = Instant::now();
    for i in 0..n {
        pipeline
            .handle_event(make_event(i), &config)
            .expect("pipeline must not error under load");
    }
    let elapsed = start.elapsed();

    // Throughput must be finite and the run must finish well within budget.
    assert!(
        elapsed < Duration::from_secs(60),
        "200k events took too long: {elapsed:?}"
    );
    let per_sec = n as f64 / elapsed.as_secs_f64().max(1e-9);
    assert!(per_sec.is_finite() && per_sec > 0.0);
}

#[test]
fn load_event_store_bounded_never_leaks() {
    // Feed far more events than capacity; the buffer must stay <= capacity and
    // accounting must balance: recorded == kept + dropped.
    let capacity = 5_000usize;
    let n = 250_000usize;
    let store = EventStore::with_capacity(capacity);

    for i in 0..n {
        store.record(make_event(i));
        // Invariant checked continuously: the buffer never exceeds capacity.
        assert!(store.len() <= capacity, "store exceeded capacity (leak)");
    }

    let stats = store.stats();
    assert_eq!(stats.total_recorded, n as u64);
    assert_eq!(
        stats.total_recorded,
        store.len() as u64 + stats.total_dropped,
        "recorded must equal kept + dropped"
    );
    assert!(stats.total_dropped >= (n - capacity) as u64);
    // Error accounting: one error every 1000 events.
    assert_eq!(stats.total_errors, (n / 1000) as u64);
}

#[test]
fn load_event_store_concurrent_writers_are_consistent() {
    // Multiple threads share one store (Arc). After all join, accounting must
    // still balance and the buffer must respect capacity. Exercises the
    // lock-free queue + stats under contention without panicking.
    let capacity = 8_000usize;
    let per_thread = 40_000usize;
    let threads = 4usize;
    let store = Arc::new(EventStore::with_capacity(capacity));

    let handles: Vec<_> = (0..threads)
        .map(|t| {
            let store = Arc::clone(&store);
            std::thread::spawn(move || {
                for i in 0..per_thread {
                    store.record(make_event(t * per_thread + i));
                }
            })
        })
        .collect();

    for h in handles {
        h.join().expect("writer thread panicked");
    }

    let total = (threads * per_thread) as u64;
    let stats = store.stats();
    assert_eq!(stats.total_recorded, total);
    assert!(store.len() <= capacity);
    assert_eq!(
        stats.total_recorded,
        store.len() as u64 + stats.total_dropped,
        "recorded must equal kept + dropped under concurrency"
    );
}

#[test]
#[ignore = "heavy soak: run explicitly with `cargo test -- --ignored`"]
fn load_heavy_soak_one_million_events() {
    // Heaviest scaled-down soak: 1,000,000 events through the pipeline. This is
    // NOT the multi-hour / 10k-concurrent production target (see module note);
    // it is a single-process throughput/stability check that must finish
    // error-free within a wide budget. Gated behind #[ignore] to keep the
    // default gate fast.
    let n = 1_000_000usize;
    let filter = Box::new(PathFilter::new().exclude("does-not-exist"));
    let processors = ProcessorPipeline::new().add_processor(Box::new(PassThroughProcessor));
    let sink = Box::new(MemorySink::new(50_000));
    let mut pipeline = EventPipeline::new(filter, processors).add_sink(sink);
    let config = Config::new(Language::Python);

    let start = Instant::now();
    for i in 0..n {
        pipeline
            .handle_event(make_event(i), &config)
            .expect("pipeline must not error under heavy load");
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(300),
        "1M events took too long: {elapsed:?}"
    );
    let per_sec = n as f64 / elapsed.as_secs_f64().max(1e-9);
    assert!(per_sec.is_finite() && per_sec > 0.0);
}
