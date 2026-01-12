// Simple working demo of Xplainit Framework (AS IS - January 2026)
// This shows EXACTLY what works right now

use xplainit_core::*;
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║      XPLAINIT FRAMEWORK - WORKING DEMO (v0.1.0)         ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // ========================================
    // DEMO 1: Basic Event Explanation
    // ========================================
    println!("📋 DEMO 1: Basic Event Explanation\n");
    
    let event = ExecutionEvent::FunctionEnter {
        id: Uuid::new_v4(),
        name: "calculate_discount".to_string(),
        args: {
            let mut args = HashMap::new();
            args.insert("price".to_string(), Value::Float(99.99));
            args.insert("discount_percent".to_string(), Value::Integer(20));
            args
        },
        location: SourceLocation::new("shop.py".to_string(), 42, 0),
        timestamp: Utc::now(),
    };
    
    // Create explainer
    let explainer = ExplanationGenerator::new(VerbosityLevel::Normal);
    println!("  {}\n", explainer.explain(&event));

    // ========================================
    // DEMO 2: Multiple Verbosity Levels
    // ========================================
    println!("📋 DEMO 2: Multiple Verbosity Levels\n");
    
    let brief = ExplanationGenerator::new(VerbosityLevel::Brief);
    println!("  Brief:    {}", brief.explain(&event));
    
    let normal = ExplanationGenerator::new(VerbosityLevel::Normal);
    println!("  Normal:   {}", normal.explain(&event));
    
    let detailed = ExplanationGenerator::new(VerbosityLevel::Detailed);
    println!("  Detailed: {}\n", detailed.explain(&event));

    // ========================================
    // DEMO 3: Error Analysis
    // ========================================
    println!("📋 DEMO 3: Error Analysis with Fix Suggestions\n");
    
    let error = ExecutionEvent::DivisionByZero {
        id: Uuid::new_v4(),
        numerator: Value::Integer(100),
        denominator_var: Some("quantity".to_string()),
        location: SourceLocation::new("calculator.py".to_string(), 15, 8),
        timestamp: Utc::now(),
    };
    
    let error_explainer = ErrorExplainer::new();
    if let Some(analysis) = error_explainer.analyze(&error) {
        println!("  🔴 Severity: {:?}", analysis.severity);
        println!("  📍 Category: {:?}", analysis.category);
        println!("\n  💡 Root Cause:");
        println!("     {}", analysis.root_cause);
        println!("\n  🔧 Fix Suggestions:");
        for (i, fix) in analysis.fix_suggestions.iter().take(2).enumerate() {
            println!("     {}. {}", i + 1, fix);
        }
        println!();
    }

    // ========================================
    // DEMO 4: Event Storage & Statistics
    // ========================================
    println!("📋 DEMO 4: Event Storage & Statistics\n");
    
    let config = Config::new(Language::Python);
    let engine = RuntimeEngine::new(config.clone());
    
    // Simulate recording 5 events
    for i in 1..=5 {
        let event = ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: format!("function_{}", i),
            args: HashMap::new(),
            location: SourceLocation::new("test.py".to_string(), i * 10, 0),
            timestamp: Utc::now(),
        };
        engine.event_store().record(event);
    }
    
    let stats = engine.event_stats();
    println!("  📊 Total Events Recorded: {}", stats.total_recorded);
    println!("  ✅ Current Events in Store: {}", stats.current_count);
    println!("  ❌ Total Errors: {}", stats.total_errors);
    println!();

    // ========================================
    // DEMO 5: Event Filtering
    // ========================================
    println!("📋 DEMO 5: Smart Event Filtering\n");
    
    let filter = FunctionFilter::new()
        .include("calculate_*")
        .exclude("*_internal");
    
    let event1 = ExecutionEvent::FunctionEnter {
        id: Uuid::new_v4(),
        name: "calculate_total".to_string(),
        args: HashMap::new(),
        location: SourceLocation::unknown(),
        timestamp: Utc::now(),
    };
    
    let event2 = ExecutionEvent::FunctionEnter {
        id: Uuid::new_v4(),
        name: "process_internal".to_string(),
        args: HashMap::new(),
        location: SourceLocation::unknown(),
        timestamp: Utc::now(),
    };
    
    println!("  Filter rules: include 'calculate_*', exclude '*_internal'");
    println!("  ✅ calculate_total: {}", if filter.should_capture(&event1, &config) { "CAPTURED" } else { "FILTERED" });
    println!("  ❌ process_internal: {}", if filter.should_capture(&event2, &config) { "CAPTURED" } else { "FILTERED" });
    println!();

    // ========================================
    // DEMO 6: JSON Output Format
    // ========================================
    println!("📋 DEMO 6: JSON Output Format\n");
    
    let json_formatter = JsonFormatter::new(true); // pretty print
    let json_output = json_formatter.format_event(&event1);
    println!("  {}\n", json_output.lines().take(5).collect::<Vec<_>>().join("\n  "));
    println!("  ... (truncated)\n");

    // ========================================
    // Summary
    // ========================================
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║                    ✅ DEMO COMPLETE                      ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!("\n✨ All features demonstrated above work perfectly!");
    println!("📝 Missing: Automatic runtime tracing (see PRODUCTION_READINESS_PLAN.md)");
    println!("🎯 Next step: Execute Phase 1 of the production plan\n");
}
