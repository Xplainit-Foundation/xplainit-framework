//! Integration tests for xplainit-core
//! Tests multi-component workflows

use xplainit_core::*;
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;

/// Test full event pipeline
#[test]
fn test_full_event_pipeline() {
    let filter = Box::new(AcceptAllFilter);
    let processors = ProcessorPipeline::new().add_processor(Box::new(PassThroughProcessor));
    let sink = Box::new(MemorySink::new(100));
    let mut pipeline = EventPipeline::new(filter, processors).add_sink(sink);
    let config = Config::new(Language::Python);
    
    let event = create_function_enter("test_func", "test.py", 10);
    pipeline.handle_event(event, &config).expect("Pipeline should handle event");
}

/// Test event filtering
#[test]
fn test_filtering_integration() {
    let config = Config::new(Language::Python);
    let filter = FunctionFilter::new().include("calculate");
    let event1 = create_function_enter("calculate", "calc.py", 10);
    let event2 = create_function_enter("other", "calc.py", 20);
    
    assert!(filter.should_capture(&event1, &config));
    assert!(!filter.should_capture(&event2, &config));
}

/// Test error analysis
#[test]
fn test_error_analysis_workflow() {
    let explainer = ErrorExplainer::new();
    let error_event = ExecutionEvent::DivisionByZero {
        id: Uuid::new_v4(),
        numerator: Value::Integer(10),
        denominator_var: Some("count".to_string()),
        location: SourceLocation::new("calc.py".to_string(), 42, 15),
        timestamp: Utc::now(),
    };
    
    let analysis = explainer.analyze(&error_event);
    assert!(analysis.is_some());
    let analysis = analysis.unwrap();
    assert_eq!(analysis.severity, ErrorSeverity::Critical);
    assert_eq!(analysis.category, ErrorCategory::Arithmetic);  // DivisionByZero is Arithmetic, not Runtime
}

/// Test formatters
#[test]
fn test_formatter_integration() {
    let events = vec![create_function_enter("main", "app.py", 1)];
    
    let text_formatter = TextFormatter::new(VerbosityLevel::Normal);
    let output = text_formatter.format_events(&events);
    assert!(!output.is_empty());
    assert!(output.contains("main"));
    
    let json_formatter = JsonFormatter::new(true);
    let output = json_formatter.format_events(&events);
    assert!(!output.is_empty());
    
    let html_formatter = HtmlFormatter::new();
    let output = html_formatter.format_events(&events);
    assert!(!output.is_empty());
}

/// Test runtime engine
#[test]
fn test_runtime_engine_lifecycle() {
    let config = Config::new(Language::Python);
    let engine = RuntimeEngine::new(config);
    
    assert_eq!(engine.state(), EngineState::Idle);
    
    // Note: start_collection requires collector and targets
    // Just test state methods exist
    assert!(!engine.is_collecting());
}

/// Test event store
#[test]
fn test_event_store_integration() {
    let store = EventStore::with_capacity(100);
    for i in 0..150 {
        store.record(create_variable_assign(&format!("var{}", i), Value::Integer(i as i64), "test.py", i));
    }
    let stats = store.stats();
    assert_eq!(stats.total_recorded, 150);
    assert!(stats.total_dropped >= 50);
}

/// Test runtime control
#[test]
fn test_runtime_control() {
    let config = Config::new(Language::Python);
    let control = RuntimeControl::new(config);
    assert!(control.is_enabled());
    control.disable();
    assert!(!control.is_enabled());
    control.enable();
    assert!(control.is_enabled());
}

/// Test explanation generator
#[test]
fn test_explanation_generator() {
    let generator = ExplanationGenerator::new(VerbosityLevel::Detailed);
    let event = create_function_enter("calculate", "math.py", 42);
    let explanation = generator.explain(&event);
    assert!(!explanation.is_empty());
    assert!(explanation.contains("calculate"));
}

/// Test processor chain
#[test]
fn test_processor_chain() {
    let mut pipeline = ProcessorPipeline::new()
        .add_processor(Box::new(PassThroughProcessor))
        .add_processor(Box::new(EnrichmentProcessor::new()));
    
    let event = create_function_enter("test", "test.py", 10);
    let result = pipeline.process(event);
    assert!(result.is_ok());
}

// Helper functions
fn create_function_enter(name: &str, file: &str, line: usize) -> ExecutionEvent {
    ExecutionEvent::FunctionEnter {
        id: Uuid::new_v4(),
        name: name.to_string(),
        args: HashMap::new(),
        location: SourceLocation::new(file.to_string(), line, 0),
        timestamp: Utc::now(),
    }
}

fn create_variable_assign(name: &str, value: Value, file: &str, line: usize) -> ExecutionEvent {
    ExecutionEvent::VariableAssign {
        id: Uuid::new_v4(),
        name: name.to_string(),
        old_value: None,
        new_value: value,
        location: SourceLocation::new(file.to_string(), line, 0),
        timestamp: Utc::now(),
    }
}
