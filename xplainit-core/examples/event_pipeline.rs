//! Event pipeline example
//! 
//! Demonstrates how to build a complete event processing pipeline
//! with filters, processors, and sinks

use xplainit_core::*;
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;

fn main() {
    println!("=== Xplainit Core - Event Pipeline Example ===\n");
    
    let config = Config::new(Language::Python);
    
    // Example 1: Simple Pipeline
    println!("--- Example 1: Simple Pipeline ---");
    simple_pipeline(&config);
    
    // Example 2: Multi-Processor Pipeline
    println!("\n--- Example 2: Multi-Processor Pipeline ---");
    multi_processor_pipeline(&config);
    
    // Example 3: Multi-Sink Pipeline
    println!("\n--- Example 3: Multi-Sink Pipeline ---");
    multi_sink_pipeline(&config);
    
    // Example 4: Production Pipeline
    println!("\n--- Example 4: Production Pipeline ---");
    production_pipeline(&config);
    
    println!("\n✓ Pipeline examples complete!");
}

fn simple_pipeline(config: &Config) {
    // Filter: Accept all events
    let filter = Box::new(AcceptAllFilter);
    
    // Processor: Pass through (no modification)
    let processors = ProcessorPipeline::new()
        .add_processor(Box::new(PassThroughProcessor));
    
    // Sink: Memory sink
    let sink = Box::new(MemorySink::new(1000));
    
    // Build pipeline
    let mut pipeline = EventPipeline::new(filter, processors)
        .add_sink(sink);
    
    println!("Pipeline: AcceptAll → PassThrough → MemorySink");
    
    // Send events through pipeline
    let events = create_sample_events();
    for event in events {
        pipeline.handle_event(event, config)
            .expect("Failed to handle event");
    }
    
    println!("✓ Processed {} events", 5);
}

fn multi_processor_pipeline(config: &Config) {
    // Filter: Only function events
    let filter = Box::new(EventTypeFilter::new().include_functions());
    
    // Processors: Chain multiple processors
    let processors = ProcessorPipeline::new()
        .add_processor(Box::new(PassThroughProcessor))
        .add_processor(Box::new(EnrichmentProcessor::new()))
        .add_processor(Box::new(DeduplicationProcessor::new()));
    
    // Sink: Console output
    let sink = Box::new(ConsoleSink);
    
    // Build pipeline
    let mut pipeline = EventPipeline::new(filter, processors)
        .add_sink(sink);
    
    println!("Pipeline: FunctionFilter → PassThrough → Enrichment → Dedup → Console");
    
    // Send events
    let events = create_sample_events();
    for event in events {
        let _ = pipeline.handle_event(event, config);
    }
}

fn multi_sink_pipeline(config: &Config) {
    // Filter: Accept all
    let filter = Box::new(AcceptAllFilter);
    
    // Processor: Enrichment
    let processors = ProcessorPipeline::new()
        .add_processor(Box::new(EnrichmentProcessor::new()));
    
    // Multiple sinks
    let memory_sink = Box::new(MemorySink::new(1000));
    let console_sink = Box::new(ConsoleSink);
    
    // Build pipeline with multiple sinks
    let mut pipeline = EventPipeline::new(filter, processors)
        .add_sink(memory_sink)
        .add_sink(console_sink);
    
    println!("Pipeline: AcceptAll → Enrichment → [MemorySink, ConsoleSink]");
    
    // Send events
    let event = ExecutionEvent::FunctionEnter {
        id: Uuid::new_v4(),
        name: "example_function".to_string(),
        args: HashMap::new(),
        location: SourceLocation::new("example.py".to_string(), 10, 0),
        timestamp: Utc::now(),
    };
    
    let _ = pipeline.handle_event(event, config);
    println!("✓ Event sent to 2 sinks");
}

fn production_pipeline(config: &Config) {
    // Filter: Advanced filtering (app code only, no stdlib)
    let filter = Box::new(AdvancedFilter::new(
        ModuleFilter::new().add_include("/app/"),
        RegexFilter::new(),
        CallStackFilter::new(20),  // Max depth 20
        PerformanceFilter::new(),
    ));
    
    // Processors: Full processing chain
    let processors = ProcessorPipeline::new()
        .add_processor(Box::new(EnrichmentProcessor::new()))
        .add_processor(Box::new(DeduplicationProcessor::new()))
        .add_processor(Box::new(RateLimitProcessor::new(1000)));  // 1000 events/sec
    
    // Sinks: Memory for analysis + console for debugging
    let memory_sink = Box::new(MemorySink::new(10000));
    let console_sink = Box::new(ConsoleSink);
    
    // Build production pipeline
    let mut pipeline = EventPipeline::new(filter, processors)
        .add_sink(memory_sink)
        .add_sink(console_sink);
    
    println!("Production Pipeline:");
    println!("  Filter: AdvancedFilter (/app/ only, max depth 20)");
    println!("  Processors: Enrichment → Dedup → RateLimit(1000/s)");
    println!("  Sinks: Memory(10K) + Console");
    
    // Simulate production events
    let app_event = ExecutionEvent::FunctionEnter {
        id: Uuid::new_v4(),
        name: "process_payment".to_string(),
        args: {
            let mut map = HashMap::new();
            map.insert("amount".to_string(), Value::Float(99.99));
            map
        },
        location: SourceLocation::new("/app/payment.py".to_string(), 45, 0),
        timestamp: Utc::now(),
    };
    
    match pipeline.handle_event(app_event, config) {
        Ok(_) => println!("✓ Production event processed successfully"),
        Err(e) => println!("✗ Error: {}", e),
    }
}

fn create_sample_events() -> Vec<ExecutionEvent> {
    vec![
        ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "main".to_string(),
            args: HashMap::new(),
            location: SourceLocation::new("app.py".to_string(), 1, 0),
            timestamp: Utc::now(),
        },
        ExecutionEvent::VariableAssign {
            id: Uuid::new_v4(),
            name: "result".to_string(),
            old_value: None,
            new_value: Value::Integer(42),
            location: SourceLocation::new("app.py".to_string(), 2, 4),
            timestamp: Utc::now(),
        },
        ExecutionEvent::ConditionalEval {
            id: Uuid::new_v4(),
            condition: "result > 0".to_string(),
            result: true,
            branch_taken: "then".to_string(),
            location: SourceLocation::new("app.py".to_string(), 3, 4),
            timestamp: Utc::now(),
        },
        ExecutionEvent::Return {
            id: Uuid::new_v4(),
            value: Some(Value::Integer(42)),
            location: SourceLocation::new("app.py".to_string(), 4, 8),
            timestamp: Utc::now(),
        },
        ExecutionEvent::FunctionExit {
            id: Uuid::new_v4(),
            name: "main".to_string(),
            return_value: Some(Value::Integer(42)),
            duration: std::time::Duration::from_millis(10),
            timestamp: Utc::now(),
        },
    ]
}
