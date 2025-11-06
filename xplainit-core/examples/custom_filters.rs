//! Example demonstrating various event filtering strategies
//! 
//! This example shows how to use different filters to control
//! which events are captured and explained.

use xplainit_core::*;
use std::collections::HashMap;
use chrono::Utc;
use uuid::Uuid;

fn main() {
    println!("=== Xplainit Core - Custom Filters Example ===\n");
    
    // Example 1: Accept All Filter
    example_accept_all();
    
    // Example 2: Function Filter
    example_function_filter();
    
    // Example 3: Event Type Filter
    example_event_type_filter();
    
    // Example 4: Depth Filter
    example_depth_filter();
    
    println!("\n✓ All filter examples complete!");
}

// Example 1: AcceptAllFilter - Captures everything
fn example_accept_all() {
    println!("--- Example 1: Accept All Filter ---");
    let filter = AcceptAllFilter;
    let config = Config::new(Language::Python);
    
    let events = create_test_events();
    
    println!("  Capturing with AcceptAllFilter:");
    for event in &events {
        if filter.should_capture(event, &config) {
            println!("    ✓ Captured: {}", event_type_name(event));
        }
    }
}

// Example 2: FunctionFilter - Filter by function name
fn example_function_filter() {
    println!("\n--- Example 2: Function Filter ---");
    let config = Config::new(Language::Python);
    let events = create_test_events();
    
    // Include specific functions
    let include_filter = FunctionFilter::new()
        .include("process_order")
        .include("calculate_total");
    
    println!("  Include filter (process_order, calculate_total):");
    for event in &events {
        if include_filter.should_capture(event, &config) {
            println!("    ✓ Captured: {}", event_type_name(event));
        }
    }
    
    // Exclude specific functions
    let exclude_filter = FunctionFilter::new()
        .exclude("internal_helper")
        .exclude("debug_log");
    
    println!("\n  Exclude filter (internal_helper, debug_log):");
    for event in &events {
        if exclude_filter.should_capture(event, &config) {
            println!("    ✓ Captured: {}", event_type_name(event));
        }
    }
}

// Example 3: EventTypeFilter - Filter by event category
fn example_event_type_filter() {
    println!("\n--- Example 3: Event Type Filter ---");
    let config = Config::new(Language::Python);
    let events = create_test_events();
    
    // Only capture errors
    let error_filter = EventTypeFilter::only_errors();
    println!("  Error filter (errors only):");
    for event in &events {
        if error_filter.should_capture(event, &config) {
            println!("    ✓ Captured: {}", event_type_name(event));
        }
    }
    
    // Only capture functions
    let function_filter = EventTypeFilter::only_functions();
    println!("\n  Function filter (functions only):");
    for event in &events {
        if function_filter.should_capture(event, &config) {
            println!("    ✓ Captured: {}", event_type_name(event));
        }
    }
}

// Example 4: DepthFilter - Limit by call stack depth
fn example_depth_filter() {
    println!("\n--- Example 4: Depth Filter ---");
    let filter = DepthFilter::new(5); // Max depth of 5
    let config = Config::new(Language::Python);
    let events = create_test_events();
    
    println!("  Depth filter (max depth: 5):");
    for event in &events {
        if filter.should_capture(event, &config) {
            println!("    ✓ Captured: {}", event_type_name(event));
        }
    }
}

// Helper: Create test events
fn create_test_events() -> Vec<ExecutionEvent> {
    let loc = SourceLocation::new("test.py".to_string(), 10, 0);
    
    vec![
        ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "process_order".to_string(),
            args: HashMap::new(),
            location: loc.clone(),
            timestamp: Utc::now(),
        },
        ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "calculate_total".to_string(),
            args: HashMap::new(),
            location: loc.clone(),
            timestamp: Utc::now(),
        },
        ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "internal_helper".to_string(),
            args: HashMap::new(),
            location: loc.clone(),
            timestamp: Utc::now(),
        },
        ExecutionEvent::DivisionByZero {
            id: Uuid::new_v4(),
            numerator: Value::Integer(100),
            denominator_var: Some("quantity".to_string()),
            location: loc.clone(),
            timestamp: Utc::now(),
        },
        ExecutionEvent::VariableAssign {
            id: Uuid::new_v4(),
            name: "total".to_string(),
            old_value: None,
            new_value: Value::Float(99.99),
            location: loc.clone(),
            timestamp: Utc::now(),
        },
    ]
}

// Helper: Get event type name
fn event_type_name(event: &ExecutionEvent) -> &str {
    match event {
        ExecutionEvent::FunctionEnter { name, .. } => name,
        ExecutionEvent::FunctionExit { name, .. } => name,
        ExecutionEvent::VariableAssign { name, .. } => name,
        ExecutionEvent::DivisionByZero { .. } => "DivisionByZero",
        ExecutionEvent::NullPointerError { .. } => "NullPointerError",
        ExecutionEvent::IndexOutOfBounds { .. } => "IndexOutOfBounds",
        _ => "Other",
    }
}
