//! Basic usage example of xplainit-core
//! 
//! This example demonstrates:
//! - Creating a runtime engine
//! - Recording events
//! - Generating explanations
//! - Formatting output

use xplainit_core::*;
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;

fn main() {
    println!("=== Xplainit Core - Basic Usage Example ===\n");
    
    // 1. Create configuration
    let config = Config::new(Language::Python);
    println!("✓ Created configuration for Python");
    
    // 2. Create runtime engine
    let engine = RuntimeEngine::new(config.clone());
    println!("✓ Created runtime engine\n");
    
    // 3. Create some example events
    println!("Recording events...");
    
    // Function enter event
    let func_enter = ExecutionEvent::FunctionEnter {
        id: Uuid::new_v4(),
        name: "calculate_total".to_string(),
        args: {
            let mut map = HashMap::new();
            map.insert("price".to_string(), Value::Float(29.99));
            map.insert("quantity".to_string(), Value::Integer(3));
            map
        },
        location: SourceLocation::new("shopping.py".to_string(), 15, 0),
        timestamp: Utc::now(),
    };
    
    // Variable assignment event
    let var_assign = ExecutionEvent::VariableAssign {
        id: Uuid::new_v4(),
        name: "subtotal".to_string(),
        old_value: None,
        new_value: Value::Float(89.97),
        location: SourceLocation::new("shopping.py".to_string(), 16, 4),
        timestamp: Utc::now(),
    };
    
    // Conditional evaluation
    let conditional = ExecutionEvent::ConditionalEval {
        id: Uuid::new_v4(),
        condition: "subtotal > 50".to_string(),
        result: true,
        branch_taken: "then".to_string(),
        location: SourceLocation::new("shopping.py".to_string(), 18, 4),
        timestamp: Utc::now(),
    };
    
    // Function exit event
    let func_exit = ExecutionEvent::FunctionExit {
        id: Uuid::new_v4(),
        name: "calculate_total".to_string(),
        return_value: Some(Value::Float(89.97)),
        duration: std::time::Duration::from_millis(5),
        timestamp: Utc::now(),
    };
    
    // Record events
    engine.event_store().record(func_enter.clone());
    engine.event_store().record(var_assign.clone());
    engine.event_store().record(conditional.clone());
    engine.event_store().record(func_exit.clone());
    
    println!("✓ Recorded 4 events\n");
    
    // 4. Generate explanations with different verbosity levels
    println!("=== Explanations ===\n");
    
    let events = vec![func_enter, var_assign, conditional, func_exit];
    
    // Brief explanations
    println!("--- Brief Verbosity ---");
    let brief_gen = ExplanationGenerator::new(VerbosityLevel::Brief);
    for event in &events {
        println!("  {}", brief_gen.explain(event));
    }
    
    println!("\n--- Normal Verbosity ---");
    let normal_gen = ExplanationGenerator::new(VerbosityLevel::Normal);
    for event in &events {
        println!("  {}", normal_gen.explain(event));
    }
    
    println!("\n--- Detailed Verbosity ---");
    let detailed_gen = ExplanationGenerator::new(VerbosityLevel::Detailed);
    for event in &events[..2] {  // Just show first 2 for brevity
        println!("  {}", detailed_gen.explain(event));
    }
    
    // 5. Format output in different formats
    println!("\n=== Formatted Output ===\n");
    
    // Text format
    println!("--- Text Format ---");
    let text_formatter = TextFormatter::new(VerbosityLevel::Normal);
    let text_output = text_formatter.format_events(&events);
    println!("{}", text_output);
    
    // JSON format (compact)
    println!("\n--- JSON Format (first event only) ---");
    let json_formatter = JsonFormatter::new(false);
    let json_output = json_formatter.format_event(&events[0]);
    println!("{}", json_output);
    
    // 6. Show event statistics
    println!("\n=== Event Statistics ===");
    let stats = engine.event_stats();
    println!("Total recorded: {}", stats.total_recorded);
    println!("Total dropped: {}", stats.total_dropped);
    println!("Current count: {}", stats.current_count);
    println!("Total errors: {}", stats.total_errors);
    
    println!("\n✓ Example complete!");
}
