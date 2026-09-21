//! Offline, std-only performance harness for the xplainit-core hot path.
//!
//! This deliberately avoids `criterion` and any external profiler so it runs
//! in a fully offline sandbox. It constructs a large volume of events and
//! times the pieces of the pipeline that FEAT-003 optimized, reporting
//! throughput in events/sec using [`std::time::Instant`].
//!
//! Run with:
//! ```text
//! cargo run --release --example bench_pipeline
//! ```
//!
//! Numbers are wall-clock and machine dependent; treat them as relative
//! before/after signals, not absolute guarantees. See `docs/PERFORMANCE.md`.

use std::collections::HashMap;
use std::time::Instant;

use chrono::Utc;
use uuid::Uuid;
use xplainit_core::{
    AcceptAllFilter, Config, EventPipeline, EventStore, ExecutionEvent, JsonFormatter, Language,
    MemorySink, OutputFormatter, PassThroughProcessor, PathFilter, ProcessorPipeline,
    SourceLocation, Value,
};

/// Number of events used for each timed section.
const N: usize = 200_000;

fn make_events(n: usize) -> Vec<ExecutionEvent> {
    let mut events = Vec::with_capacity(n);
    for i in 0..n {
        let mut args = HashMap::with_capacity(2);
        args.insert("x".to_string(), Value::Integer(i as i64));
        args.insert("label".to_string(), Value::String("payload".to_string()));
        events.push(ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "hot_function".to_string(),
            args,
            location: SourceLocation {
                file: "app/module.py".to_string(),
                line: (i % 500) + 1,
                column: 0,
                offset: 0,
            },
            timestamp: Utc::now(),
        });
    }
    events
}

fn report(label: &str, n: usize, elapsed: std::time::Duration) {
    let secs = elapsed.as_secs_f64();
    let per_sec = if secs > 0.0 {
        n as f64 / secs
    } else {
        f64::INFINITY
    };
    println!("{label:<28} {n:>9} events in {secs:>8.4}s  =>  {per_sec:>14.0} events/sec",);
}

fn bench_filter(events: &[ExecutionEvent]) {
    use xplainit_core::EventFilter;
    let config = Config::new(Language::Python);
    // PathFilter exercises the location borrow refactor (no SourceLocation
    // clone per event). An exclude pattern that never matches keeps the full
    // pattern-matching path warm.
    let filter = PathFilter::new().exclude("does/not/match");

    let start = Instant::now();
    let mut kept = 0usize;
    for e in events {
        if filter.should_capture(e, &config) {
            kept += 1;
        }
    }
    let elapsed = start.elapsed();
    std::hint::black_box(kept);
    report("path_filter.should_capture", events.len(), elapsed);
}

fn bench_store(events: &[ExecutionEvent]) {
    let store = EventStore::with_capacity(events.len().max(1));
    let start = Instant::now();
    for e in events {
        store.record(e.clone());
    }
    let elapsed = start.elapsed();
    report("event_store.record", events.len(), elapsed);
}

fn bench_pipeline(events: &[ExecutionEvent]) {
    let filter = Box::new(AcceptAllFilter);
    let processors = ProcessorPipeline::new().add_processor(Box::new(PassThroughProcessor));
    let sink = Box::new(MemorySink::new(events.len().max(1)));
    let mut pipeline = EventPipeline::new(filter, processors).add_sink(sink);
    let config = Config::new(Language::Python);

    let start = Instant::now();
    for e in events {
        let _ = pipeline.handle_event(e.clone(), &config);
    }
    let elapsed = start.elapsed();
    report("pipeline.handle_event", events.len(), elapsed);
}

fn bench_memory_sink_overflow(events: &[ExecutionEvent]) {
    use xplainit_core::EventSink;
    // Small capacity forces an eviction on almost every write, exercising the
    // VecDeque O(1) pop_front instead of the old Vec::remove(0) O(n) shift.
    let mut sink = MemorySink::new(1_000);
    let start = Instant::now();
    for e in events {
        let _ = sink.write(e);
    }
    let elapsed = start.elapsed();
    report("memory_sink.write(overflow)", events.len(), elapsed);
}

fn bench_json_format(events: &[ExecutionEvent]) {
    let formatter = JsonFormatter::new(false);
    let start = Instant::now();
    let out = formatter.format_events(events);
    let elapsed = start.elapsed();
    std::hint::black_box(out.len());
    report("json.format_events", events.len(), elapsed);
}

fn main() {
    println!("xplainit-core offline benchmark harness (std::time::Instant)");
    println!("N = {N} events per section; release build recommended.\n");

    let events = make_events(N);

    bench_filter(&events);
    bench_store(&events);
    bench_pipeline(&events);
    bench_memory_sink_overflow(&events);
    bench_json_format(&events);

    println!("\nDone. Numbers are wall-clock and machine dependent.");
}
