//! Error analysis example
//! 
//! Demonstrates how to use ErrorExplainer to analyze errors
//! and get actionable fix suggestions

use xplainit_core::*;
use chrono::Utc;
use uuid::Uuid;

fn main() {
    println!("=== Xplainit Core - Error Analysis Example ===\n");
    
    let explainer = ErrorExplainer::new();
    
    // Example 1: Division by Zero
    println!("--- Example 1: Division by Zero ---");
    let div_zero = ExecutionEvent::DivisionByZero {
        id: Uuid::new_v4(),
        numerator: Value::Integer(100),
        denominator_var: Some("user_input".to_string()),
        location: SourceLocation::new("calculator.py".to_string(), 42, 15),
        timestamp: Utc::now(),
    };
    
    if let Some(analysis) = explainer.analyze(&div_zero) {
        print_analysis(&analysis);
    }
    
    // Example 2: Null Pointer Error
    println!("\n--- Example 2: Null Pointer Error ---");
    let null_ptr = ExecutionEvent::NullPointerError {
        id: Uuid::new_v4(),
        variable: "customer".to_string(),
        operation: "accessing property 'name'".to_string(),
        location: SourceLocation::new("order.py".to_string(), 78, 8),
        timestamp: Utc::now(),
    };
    
    if let Some(analysis) = explainer.analyze(&null_ptr) {
        print_analysis(&analysis);
    }
    
    // Example 3: Index Out of Bounds
    println!("\n--- Example 3: Index Out of Bounds ---");
    let index_error = ExecutionEvent::IndexOutOfBounds {
        id: Uuid::new_v4(),
        index: 10,
        size: 5,
        collection: "items_list".to_string(),
        location: SourceLocation::new("data.py".to_string(), 125, 20),
        timestamp: Utc::now(),
    };
    
    if let Some(analysis) = explainer.analyze(&index_error) {
        print_analysis(&analysis);
    }
    
    // Example 4: Type Error
    println!("\n--- Example 4: Type Error ---");
    let type_error = ExecutionEvent::TypeError {
        id: Uuid::new_v4(),
        expected: "int".to_string(),
        got: "str".to_string(),
        value: Value::String("hello".to_string()),
        operation: "addition".to_string(),
        location: SourceLocation::new("math.py".to_string(), 55, 12),
        timestamp: Utc::now(),
    };
    
    if let Some(analysis) = explainer.analyze(&type_error) {
        print_analysis(&analysis);
    }
    
    // Example 5: Stack Overflow (Infinite Recursion)
    println!("\n--- Example 5: Stack Overflow ---");
    let stack_overflow = ExecutionEvent::StackOverflow {
        id: Uuid::new_v4(),
        recursion_depth: 1000,
        function: "factorial".to_string(),
        location: SourceLocation::new("recursive.py".to_string(), 10, 4),
        timestamp: Utc::now(),
    };
    
    if let Some(analysis) = explainer.analyze(&stack_overflow) {
        print_analysis(&analysis);
    }
    
    println!("\n✓ Error analysis examples complete!");
}

fn print_analysis(analysis: &ErrorAnalysis) {
    println!("Severity: {:?}", analysis.severity);
    println!("Category: {:?}", analysis.category);
    println!("\nRoot Cause:");
    println!("  {}", analysis.root_cause);
    
    if !analysis.leading_events.is_empty() {
        println!("\nLeading Events:");
        for (i, event) in analysis.leading_events.iter().enumerate() {
            println!("  {}. {}", i + 1, event);
        }
    }
    
    println!("\nFix Suggestions:");
    for (i, suggestion) in analysis.fix_suggestions.iter().enumerate() {
        println!("  {}. {}", i + 1, suggestion);
    }
    
    if !analysis.prevention_tips.is_empty() {
        println!("\nPrevention Tips:");
        for (i, tip) in analysis.prevention_tips.iter().enumerate() {
            println!("  {}. {}", i + 1, tip);
        }
    }
    
    if !analysis.resources.is_empty() {
        println!("\nResources:");
        for (i, resource) in analysis.resources.iter().enumerate() {
            println!("  {}. {}", i + 1, resource);
        }
    }
}
