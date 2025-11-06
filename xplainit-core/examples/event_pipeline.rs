//! Event Pipeline Example - Demonstrates Filter → Processor → Sink Architecture
//!
//! This example shows how to build complete event processing pipelines:
//! 1. Simple pipeline with single processor and sink
//! 2. Multi-processor chain with deduplication and rate limiting
//! 3. Multi-sink output (console + memory)
//! 4. Production-ready configuration
//!
//! Run with: cargo run --example event_pipeline --release

use xplainit_core::{
    Config, ExecutionEvent, Language, SourceLocation, OutputFormat, Value, Verbosity,
    filter::{AcceptAllFilter, EventTypeFilter},
    processor::{DeduplicationProcessor, EnrichmentProcessor, PassThroughProcessor, ProcessorPipeline, RateLimitProcessor},
    sink::{ConsoleSink, MemorySink, MultiSink},
    pipeline::EventPipeline,
};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// Generate sample events for testing
fn generate_sample_events() -> Vec<ExecutionEvent> {
    let base_location = SourceLocation {
        file: "example.py".into(),
        line: 10,
        column: 4,
        offset: 100,
    };

    vec![
        ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            location: base_location.clone(),
            name: "process_data".into(),
            args: {
                let mut map = HashMap::new();
                map.insert("data".into(), Value::String("vec![1,2,3]".into()));
                map
            },
        },
        ExecutionEvent::VariableAssign {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "example.py".into(),
                line: 11,
                column: 8,
                offset: 120,
            },
            name: "result".into(),
            old_value: None,
            new_value: Value::Integer(0),
        },
        ExecutionEvent::LoopEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "example.py".into(),
                line: 12,
                column: 8,
                offset: 140,
            },
            loop_type: "for".into(),
            condition: Some("item in data".into()),
        },
        ExecutionEvent::VariableAssign {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "example.py".into(),
                line: 13,
                column: 12,
                offset: 160,
            },
            name: "result".into(),
            old_value: Some(Value::Integer(0)),
            new_value: Value::Integer(3),
        },
        ExecutionEvent::DivisionByZero {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "example.py".into(),
                line: 14,
                column: 12,
                offset: 180,
            },
            numerator: Value::Integer(10),
            denominator_var: Some("zero".into()),
        },
        ExecutionEvent::FunctionExit {
            id: Uuid::new_v4(),
            name: "process_data".into(),
            return_value: Some(Value::Null),
            duration: Duration::from_millis(5),
            timestamp: Utc::now(),
        },
    ]
}

/// Example 1: Simple pipeline with pass-through processing
fn example_simple_pipeline() {
    println!("\n=== Example 1: Simple Pipeline ===\n");

    let filter = Box::new(AcceptAllFilter);
    let processors = ProcessorPipeline::new()
        .add_processor(Box::new(PassThroughProcessor));
    let sink = Box::new(ConsoleSink::new(OutputFormat::Console));

    let mut pipeline = EventPipeline::new(filter, processors)
        .add_sink(sink);

    let config = Config::new(Language::Python)
        .with_verbosity(Verbosity::Detailed);

    println!("Processing events with simple pass-through pipeline...\n");
    
    for event in generate_sample_events().into_iter().take(3) {
        let _ = pipeline.handle_event(event, &config);
    }
}

/// Example 2: Multi-processor pipeline with deduplication and rate limiting
fn example_multi_processor_pipeline() {
    println!("\n=== Example 2: Multi-Processor Pipeline ===\n");

    let filter = Box::new(AcceptAllFilter);
    
    // Build processor chain: Enrichment → Deduplication → Rate Limiting
    let processors = ProcessorPipeline::new()
        .add_processor(Box::new(EnrichmentProcessor::new()))
        .add_processor(Box::new(DeduplicationProcessor::new(100)))
        .add_processor(Box::new(RateLimitProcessor::new(1000)));

    let sink = Box::new(ConsoleSink::new(OutputFormat::Console));

    let mut pipeline = EventPipeline::new(filter, processors)
        .add_sink(sink);

    let config = Config::new(Language::Python)
        .with_verbosity(Verbosity::Normal);

    println!("Processing events with enrichment + deduplication + rate limiting...\n");
    
    let events = generate_sample_events();
    
    // Process first batch
    for event in events.iter().take(4) {
        let _ = pipeline.handle_event(event.clone(), &config);
    }

    // Process duplicate (should be filtered by deduplication)
    println!("\n[Processing duplicate event - should be deduplicated]\n");
    let _ = pipeline.handle_event(events[0].clone(), &config);
}

/// Example 3: Multi-sink output (Console + Memory)
fn example_multi_sink_pipeline() {
    println!("\n=== Example 3: Multi-Sink Pipeline ===\n");

    let filter = Box::new(EventTypeFilter::only_errors());
    let processors = ProcessorPipeline::new()
        .add_processor(Box::new(EnrichmentProcessor::new()));

    // Create memory sink to capture events
    let memory = MemorySink::new(100);
    let memory_clone = memory.clone();

    // Multi-sink: Console (JSON) + Memory
    let console = Box::new(ConsoleSink::new(OutputFormat::Json));
    let mem_sink = Box::new(memory_clone);

    let mut pipeline = EventPipeline::new(filter, processors)
        .add_sink(console)
        .add_sink(mem_sink);

    let config = Config::new(Language::Python)
        .with_verbosity(Verbosity::Detailed);

    println!("Processing events with error-only filter to console + memory...\n");
    
    for event in generate_sample_events() {
        let _ = pipeline.handle_event(event, &config);
    }

    // Check memory sink
    let captured = memory.get_events();
    println!("\n[Memory sink captured {} error events]", captured.len());
}

/// Example 4: Production-ready configuration
fn example_production_pipeline() {
    println!("\n=== Example 4: Production Pipeline ===\n");

    let filter = Box::new(EventTypeFilter::only_errors());
    
    // Production processor chain: optimize for performance
    let processors = ProcessorPipeline::new()
        .add_processor(Box::new(DeduplicationProcessor::new(1000)))
        .add_processor(Box::new(RateLimitProcessor::new(500)));

    // Multi-sink for production: JSON console + Memory buffer
    let multi_sink = MultiSink::new()
        .add_sink(Box::new(ConsoleSink::new(OutputFormat::Json)))
        .add_sink(Box::new(MemorySink::new(10000)));

    let mut pipeline = EventPipeline::new(filter, processors)
        .add_sink(Box::new(multi_sink));

    let config = Config::new(Language::Python)
        .with_verbosity(Verbosity::Brief)
        .with_max_depth(10);

    println!("Production config: Error-only, Dedup(1000), RateLimit(500/s), JSON output\n");
    
    for event in generate_sample_events() {
        let _ = pipeline.handle_event(event, &config);
    }

    println!("\n[Production pipeline processed events successfully]");
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         Xplainit Event Pipeline Examples                  ║");
    println!("║  Filter → Processor → Sink Architecture Demonstrations    ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    example_simple_pipeline();
    example_multi_processor_pipeline();
    example_multi_sink_pipeline();
    example_production_pipeline();

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  All pipeline examples completed successfully!             ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
}
