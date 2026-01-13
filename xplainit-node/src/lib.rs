//! Xplainit Node.js Bindings
//!
//! This module provides Node.js/JavaScript bindings for the Xplainit Framework
//! using Neon (N-API) for native Node.js addon development.

use neon::prelude::*;
use xplainit_core::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;
use chrono::Utc;
use uuid::Uuid;

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
        rt.event_store().clear();
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

/// Record function entry (called from JavaScript tracer)
fn on_function_enter(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let function_name = cx.argument::<JsString>(0)?.value(&mut cx);
    let args_obj = cx.argument::<JsObject>(1)?;
    let file_path = cx.argument::<JsString>(2)?.value(&mut cx);
    let line_number = cx.argument::<JsNumber>(3)?.value(&mut cx) as u32;
    
    let global_runtime = RUNTIME.lock().unwrap();
    
    if let Some(runtime) = &*global_runtime {
        let rt = runtime.lock().unwrap();
        
        // Convert JavaScript object to HashMap
        let mut args = std::collections::HashMap::new();
        let keys = args_obj.get_own_property_names(&mut cx)?;
        let keys_vec = keys.to_vec(&mut cx)?;
        
        for key_val in keys_vec {
            if let Ok(key) = key_val.downcast::<JsString, _>(&mut cx) {
                let key_str = key.value(&mut cx);
                let val: Handle<JsValue> = args_obj.get(&mut cx, key_str.as_str())?;
                let val_str = val.to_string(&mut cx)?.value(&mut cx);
                args.insert(key_str, xplainit_core::Value::String(val_str));
            }
        }
        
        // Create FunctionEnter event
        let event = ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: function_name,
            args,
            location: SourceLocation::new(file_path, line_number as usize, 0),
            timestamp: Utc::now(),
        };
        
        rt.event_store().record(event);
    }
    
    Ok(cx.boolean(true))
}

/// Record function exit (called from JavaScript tracer)
fn on_function_exit(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let function_name = cx.argument::<JsString>(0)?.value(&mut cx);
    let return_value_str = cx.argument::<JsString>(1)?.value(&mut cx);
    
    let global_runtime = RUNTIME.lock().unwrap();
    
    if let Some(runtime) = &*global_runtime {
        let rt = runtime.lock().unwrap();
        
        // Create FunctionExit event
        let event = ExecutionEvent::FunctionExit {
            id: Uuid::new_v4(),
            name: function_name,
            return_value: Some(xplainit_core::Value::String(return_value_str)),
            duration: Duration::from_secs(0),
            timestamp: Utc::now(),
        };
        
        rt.event_store().record(event);
    }
    
    Ok(cx.boolean(true))
}

/// Record exception (called from JavaScript tracer)
fn on_exception(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let error_type = cx.argument::<JsString>(0)?.value(&mut cx);
    let error_message = cx.argument::<JsString>(1)?.value(&mut cx);
    let file_path = cx.argument::<JsString>(2)?.value(&mut cx);
    let line_number = cx.argument::<JsNumber>(3)?.value(&mut cx) as u32;
    
    let global_runtime = RUNTIME.lock().unwrap();
    
    if let Some(runtime) = &*global_runtime {
        let rt = runtime.lock().unwrap();
        
        // Create Exception event
        let event = ExecutionEvent::Exception {
            id: Uuid::new_v4(),
            error_type,
            message: error_message,
            location: SourceLocation::new(file_path, line_number as usize, 0),
            stack_trace: vec![],
            caught: false,
            timestamp: Utc::now(),
        };
        
        rt.event_store().record(event);
    }
    
    Ok(cx.boolean(true))
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
    
    // Export tracer callback functions
    cx.export_function("onFunctionEnter", on_function_enter)?;
    cx.export_function("onFunctionExit", on_function_exit)?;
    cx.export_function("onException", on_exception)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // This test just ensures the Neon module compiles successfully
    }
}
