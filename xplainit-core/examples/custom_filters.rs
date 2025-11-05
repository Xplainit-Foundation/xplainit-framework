//! Custom filtering example
//! 
//! Demonstrates how to create and combine filters to control
//! which events are captured

use xplainit_core::*;
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;

fn main() {
    println!("=== Xplainit Core - Custom Filtering Example ===\n");
    
    let config = Config::new(Language::Python);
    
    // Create test events
    let events = create_test_events();
    
    // Example 1: AcceptAllFilter
    println!("--- Example 1: AcceptAllFilter ---");
    let accept_all = AcceptAllFilter;
    let captured = events.iter()
        .filter(|e| accept_all.should_capture(e, &config))
        .count();
    println!("Captured {} out of {} events", captured, events.len());
    
    // Example 2: FunctionFilter (include specific functions)
    println!("\n--- Example 2: FunctionFilter (include) ---");
    let func_filter = FunctionFilter::new()
        .include("process_order")
        .include("calculate_tax");
    
    println!("Including only: process_order, calculate_tax");
    for event in &events {
        if func_filter.should_capture(event, &config) {
            if let ExecutionEvent::FunctionEnter { name, .. } = event {
                println!("  ✓ Captured: {}", name);
            }
        }
    }
    
    // Example 3: FunctionFilter (exclude specific functions)
    println!("\n--- Example 3: FunctionFilter (exclude) ---");
    let func_filter = FunctionFilter::new()
        .exclude("internal_helper")
        .exclude("debug_log");
    
    println!("Excluding: internal_helper, debug_log");
    let captured = events.iter()
        .filter(|e| func_filter.should_capture(e, &config))
        .count();
    println!("Captured {} out of {} events", captured, events.len());
    
    // Example 4: ModuleFilter (include specific paths)
    println!("\n--- Example 4: ModuleFilter (include paths) ---");
    let module_filter = ModuleFilter::new()
        .add_include("/app/");
    
    let app_event = ExecutionEvent::FunctionEnter {
        id: Uuid::new_v4(),
        name: "app_function".to_string(),
        args: HashMap::new(),
        location: SourceLocation::new("/app/main.py".to_string(), 10, 0),
        timestamp: Utc::now(),
    };
    
    let lib_event = ExecutionEvent::FunctionEnter {
        id: Uuid::new_v4(),
        name: "lib_function".to_string(),
        args: HashMap::new(),
        location: SourceLocation::new("/usr/lib/python3.9/os.py".to_string(), 100, 0),
        timestamp: Utc::now(),
    };
    
    println!("  /app/main.py: {}", if module_filter.should_capture(&app_event, &config) { "✓ Captured" } else { "✗ Filtered" });
    println!("  /usr/lib/python3.9/os.py: {}", if module_filter.should_capture(&lib_event, &config) { "✓ Captured" } else { "✗ Filtered" });
    
    // Example 5: EventTypeFilter
    println!("\n--- Example 5: EventTypeFilter ---");
    let error_filter = EventTypeFilter::new()
        .include_errors();
    
    println!("Including only error events:");
    for event in &events {
        if error_filter.should_capture(event, &config) {
            println!("  ✓ Captured error: {:?}", event_type_name(event));
        }
    }
    
    // Example 6: AdvancedFilter (combining multiple filters)
    println!("\n--- Example 6: AdvancedFilter (combined) ---");
    let advanced = AdvancedFilter::new(
        ModuleFilter::new().add_include("/app/"),
        RegexFilter::new(),
        CallStackFilter::new(10),  // Max depth 10
        PerformanceFilter::new(),
    );
    
    println!("Using AdvancedFilter with:");
    println!("  - ModuleFilter: /app/ only");
    println!("  - CallStackFilter: max depth 10");
    println!("  - PerformanceFilter: sampling");
    
    if advanced.should_capture(&app_event, &config) {
        println!("  ✓ App event captured");
    }
    
    // Example 7: RegexFilter (pattern matching)
    println!("\n--- Example 7: RegexFilter (pattern matching) ---");
    let regex_filter = RegexFilter::new()
        .add_include_pattern("^process_")  // Functions starting with "process_"
        .add_include_pattern("_handler$");  // Functions ending with "_handler"
    
    let test_functions = vec![
        "process_payment",
        "process_refund",
        "click_handler",
        "calculate_total",
        "submit_handler",
    ];
    
    println!("Matching functions with: ^process_ or _handler$");
    for func_name in test_functions {
        let event = ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: func_name.to_string(),
            args: HashMap::new(),
            location: SourceLocation::new("test.py".to_string(), 1, 0),
            timestamp: Utc::now(),
        };
        
        let matched = regex_filter.should_capture(&event, &config);
        println!("  {} {}", func_name, if matched { "✓" } else { "✗" });
    }
    
    println!("\n✓ Filtering examples complete!");
}

fn create_test_events() -> Vec<ExecutionEvent> {
    vec![
        ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "process_order".to_string(),
            args: HashMap::new(),
            location: SourceLocation::new("orders.py".to_string(), 10, 0),
            timestamp: Utc::now(),
        },
        ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "calculate_tax".to_string(),
            args: HashMap::new(),
            location: SourceLocation::new("tax.py".to_string(), 25, 0),
            timestamp: Utc::now(),
        },
        ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "internal_helper".to_string(),
            args: HashMap::new(),
            location: SourceLocation::new("utils.py".to_string(), 5, 0),
            timestamp: Utc::now(),
        },
        ExecutionEvent::DivisionByZero {
            id: Uuid::new_v4(),
            numerator: Value::Integer(100),
            denominator_var: Some("count".to_string()),
            location: SourceLocation::new("calc.py".to_string(), 42, 15),
            timestamp: Utc::now(),
        },
    ]
}

fn event_type_name(event: &ExecutionEvent) -> &'static str {
    match event {
        ExecutionEvent::FunctionEnter { .. } => "FunctionEnter",
        ExecutionEvent::FunctionExit { .. } => "FunctionExit",
        ExecutionEvent::DivisionByZero { .. } => "DivisionByZero",
        ExecutionEvent::NullPointerError { .. } => "NullPointerError",
        ExecutionEvent::IndexOutOfBounds { .. } => "IndexOutOfBounds",
        ExecutionEvent::TypeError { .. } => "TypeError",
        ExecutionEvent::StackOverflow { .. } => "StackOverflow",
        _ => "Other",
    }
}
