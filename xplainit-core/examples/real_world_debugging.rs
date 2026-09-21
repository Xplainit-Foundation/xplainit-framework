//! Real-world debugging example
//!
//! This example walks through a realistic debugging session end to end using
//! only the public `xplainit-core` API. It simulates tracing a small e-commerce
//! "checkout" pipeline that computes an average price per item, then trips over a
//! division-by-zero when an order arrives with zero items.
//!
//! It demonstrates the full loop a real integration performs:
//! 1. Record function-enter / variable / function-exit / exception events into an
//!    `EventStore` as the program "runs".
//! 2. Read the recorded events back and turn them into natural-language
//!    explanations with `ExplanationGenerator` at different verbosity levels.
//! 3. Feed the failing event to `ErrorExplainer` to get root cause, fix
//!    suggestions, and prevention tips - the kind of output a developer would
//!    actually act on.
//!
//! Run with: cargo run --release --example real_world_debugging -p xplainit-core

use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;
use xplainit_core::*;

fn main() {
    println!("=== Xplainit Core - Real-World Debugging Example ===\n");
    println!("Scenario: a checkout service computes the average price per item.");
    println!("A buggy order arrives with 0 items, causing a division-by-zero.\n");

    // The store is what a language binding fills as the traced program runs.
    let store = EventStore::new();

    // Simulate the execution of `checkout(order)` and record what happens.
    simulate_checkout(&store);

    // Snapshot the recorded events (non-draining) so we can replay them.
    let events = store.snapshot();
    println!("Recorded {} events during the traced run.\n", events.len());

    // --- Step 1: Replay the execution as a human-readable narrative ----------
    println!("--- Execution Trace (Normal verbosity) ---");
    let narrator = ExplanationGenerator::new(VerbosityLevel::Normal);
    for event in &events {
        println!("  {}", narrator.explain(event));
    }

    // --- Step 2: Zoom in on the failure with root-cause analysis -------------
    println!("\n--- Root-Cause Analysis of the Failure ---");
    let explainer = ErrorExplainer::new();
    let mut analyzed_any = false;
    for event in &events {
        if event.is_error() {
            if let Some(analysis) = explainer.analyze(event) {
                print_analysis(&analysis);
                analyzed_any = true;
            }
        }
    }
    if !analyzed_any {
        println!("  No error events were captured.");
    }

    // --- Step 3: Summarise what the trace tells us ---------------------------
    println!("\n--- Session Summary ---");
    let stats = store.stats();
    println!("  Total events recorded: {}", stats.total_recorded);
    println!("  Error events:          {}", stats.total_errors);
    println!("  Events still buffered: {}", stats.current_count);

    println!("\n✓ Real-world debugging example complete!");
}

/// Simulate tracing `checkout(order)` and record events into the store.
///
/// The "program" being traced is roughly:
/// ```python
/// def checkout(order):
///     total = sum(item.price for item in order)   # 89.97
///     count = len(order)                           # 0  <-- bug: empty order
///     average = total / count                      # ZeroDivisionError
///     return average
/// ```
fn simulate_checkout(store: &EventStore) {
    let file = "checkout.py".to_string();

    // Entering checkout(order=[])
    let mut args = HashMap::new();
    args.insert("order".to_string(), Value::Array(vec![]));
    store.record(ExecutionEvent::FunctionEnter {
        id: Uuid::new_v4(),
        name: "checkout".to_string(),
        args,
        location: SourceLocation::new(file.clone(), 1, 0),
        timestamp: Utc::now(),
    });

    // total = 89.97
    store.record(ExecutionEvent::VariableAssign {
        id: Uuid::new_v4(),
        name: "total".to_string(),
        old_value: None,
        new_value: Value::Float(89.97),
        location: SourceLocation::new(file.clone(), 2, 4),
        timestamp: Utc::now(),
    });

    // count = 0  (the root of the problem)
    store.record(ExecutionEvent::VariableAssign {
        id: Uuid::new_v4(),
        name: "count".to_string(),
        old_value: None,
        new_value: Value::Integer(0),
        location: SourceLocation::new(file.clone(), 3, 4),
        timestamp: Utc::now(),
    });

    // average = total / count  ->  division by zero
    store.record(ExecutionEvent::DivisionByZero {
        id: Uuid::new_v4(),
        numerator: Value::Float(89.97),
        denominator_var: Some("count".to_string()),
        location: SourceLocation::new(file.clone(), 4, 14),
        timestamp: Utc::now(),
    });

    // The raised exception unwinds out of checkout without a clean return.
    store.record(ExecutionEvent::FunctionExit {
        id: Uuid::new_v4(),
        name: "checkout".to_string(),
        return_value: None,
        duration: Duration::from_micros(120),
        timestamp: Utc::now(),
    });
}

/// Pretty-print an `ErrorAnalysis` the way a debugging tool would surface it.
fn print_analysis(analysis: &ErrorAnalysis) {
    println!("  Severity: {:?}", analysis.severity);
    println!("  Category: {:?}", analysis.category);
    println!("  Root cause: {}", analysis.root_cause);

    if !analysis.fix_suggestions.is_empty() {
        println!("  Fix suggestions:");
        for (i, suggestion) in analysis.fix_suggestions.iter().enumerate() {
            println!("    {}. {}", i + 1, suggestion);
        }
    }

    if !analysis.prevention_tips.is_empty() {
        println!("  Prevention tips:");
        for (i, tip) in analysis.prevention_tips.iter().enumerate() {
            println!("    {}. {}", i + 1, tip);
        }
    }
}
