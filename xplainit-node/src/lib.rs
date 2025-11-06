//! Xplainit Node.js Bindings
//!
//! This module provides Node.js/JavaScript bindings for the Xplainit Framework
//! using Neon (N-API) for native Node.js addon development.

use neon::prelude::*;
use xplainit_core::{
    Config, Language, RuntimeEngine, Verbosity, OutputFormat,
};
use std::sync::{Arc, Mutex};

/// Global runtime instance
static RUNTIME: Mutex<Option<Arc<Mutex<RuntimeEngine>>>> = Mutex::new(None);

/// Enable tracing with the provided configuration
fn enable(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let config = Config::new(Language::JavaScript)
        .with_verbosity(Verbosity::Normal)
        .with_output_format(OutputFormat::Json);
    
    let runtime = RuntimeEngine::new(config);
    
    let mut global_runtime = RUNTIME.lock().unwrap();
    *global_runtime = Some(Arc::new(Mutex::new(runtime)));
    
    Ok(cx.boolean(true))
}

/// Disable tracing
fn disable(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let mut global_runtime = RUNTIME.lock().unwrap();
    *global_runtime = None;
    
    Ok(cx.boolean(true))
}

/// Check if tracing is enabled
fn is_enabled(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let global_runtime = RUNTIME.lock().unwrap();
    Ok(cx.boolean(global_runtime.is_some()))
}

/// Get captured events as JSON string
fn get_events(mut cx: FunctionContext) -> JsResult<JsString> {
    let global_runtime = RUNTIME.lock().unwrap();
    
    if let Some(runtime) = &*global_runtime {
        let rt = runtime.lock().unwrap();
        let events = rt.get_events();
        
        let json = serde_json::to_string(&events)
            .unwrap_or_else(|_| "[]".to_string());
        
        Ok(cx.string(json))
    } else {
        Ok(cx.string("[]"))
    }
}

/// Clear all captured events
fn clear_events(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let global_runtime = RUNTIME.lock().unwrap();
    
    if let Some(runtime) = &*global_runtime {
        let rt = runtime.lock().unwrap();
        rt.clear_events();
    }
    
    Ok(cx.boolean(true))
}

/// Get statistics about captured events
fn get_statistics(mut cx: FunctionContext) -> JsResult<JsObject> {
    let global_runtime = RUNTIME.lock().unwrap();
    
    let stats = cx.empty_object();
    
    if let Some(runtime) = &*global_runtime {
        let rt = runtime.lock().unwrap();
        let events = rt.get_events();
        
        let total = cx.number(events.len() as f64);
        stats.set(&mut cx, "total_events", total)?;
        
        // Count event types
        let mut function_calls = 0;
        let mut variable_assigns = 0;
        let mut errors = 0;
        
        for event in events {
            match event {
                xplainit_core::ExecutionEvent::FunctionEnter { .. } |
                xplainit_core::ExecutionEvent::FunctionExit { .. } => function_calls += 1,
                xplainit_core::ExecutionEvent::VariableAssign { .. } |
                xplainit_core::ExecutionEvent::VariableDeclaration { .. } => variable_assigns += 1,
                xplainit_core::ExecutionEvent::DivisionByZero { .. } |
                xplainit_core::ExecutionEvent::NullPointerError { .. } |
                xplainit_core::ExecutionEvent::IndexOutOfBounds { .. } => errors += 1,
                _ => {}
            }
        }
        
        let fn_count = cx.number(function_calls as f64);
        stats.set(&mut cx, "function_calls", fn_count)?;
        
        let var_count = cx.number(variable_assigns as f64);
        stats.set(&mut cx, "variable_operations", var_count)?;
        
        let err_count = cx.number(errors as f64);
        stats.set(&mut cx, "errors", err_count)?;
    } else {
        let zero = cx.number(0.0);
        stats.set(&mut cx, "total_events", zero)?;
        stats.set(&mut cx, "function_calls", zero)?;
        stats.set(&mut cx, "variable_operations", zero)?;
        stats.set(&mut cx, "errors", zero)?;
    }
    
    Ok(stats)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    // Export module-level functions
    cx.export_function("enable", enable)?;
    cx.export_function("disable", disable)?;
    cx.export_function("isEnabled", is_enabled)?;
    cx.export_function("getEvents", get_events)?;
    cx.export_function("clearEvents", clear_events)?;
    cx.export_function("getStatistics", get_statistics)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // This test just ensures the Neon module compiles successfully
    }
}
