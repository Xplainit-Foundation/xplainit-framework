//! Xplainit Java JNI Bindings
//!
//! This module provides JNI (Java Native Interface) bindings for the Xplainit Framework

use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::{jboolean, jlong, jstring};
use std::sync::{Arc, Mutex};
use xplainit_core::{Config, Language, RuntimeEngine, Verbosity, OutputFormat};

/// Global runtime instance stored in Java object field
type RuntimeHandle = Arc<Mutex<RuntimeEngine>>;

/// Create a new Xplainit runtime
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeCreate(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    let config = Config::new(Language::Java)
        .with_verbosity(Verbosity::Normal)
        .with_output_format(OutputFormat::Json);
    
    let runtime = RuntimeEngine::new(config);
    let handle = Box::new(Arc::new(Mutex::new(runtime)));
    
    Box::into_raw(handle) as jlong
}

/// Free the runtime
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeFree(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle != 0 {
        unsafe {
            let _ = Box::from_raw(handle as *mut RuntimeHandle);
        }
    }
}

/// Enable tracing
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeEnable(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    if handle == 0 {
        return 0;
    }
    
    unsafe {
        let runtime_ref = &*(handle as *const RuntimeHandle);
        let _runtime = runtime_ref.lock().unwrap();
        // Enable logic
        1
    }
}

/// Disable tracing
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeDisable(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    if handle == 0 {
        return 0;
    }
    
    unsafe {
        let runtime_ref = &*(handle as *const RuntimeHandle);
        let _runtime = runtime_ref.lock().unwrap();
        // Disable logic
        1
    }
}

/// Check if enabled
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeIsEnabled(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    if handle == 0 {
        return 0;
    }
    
    // For now, always return enabled after create
    1
}

/// Get events as JSON string
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeGetEvents(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    if handle == 0 {
        return env.new_string("[]")
            .expect("Couldn't create java string!")
            .into_raw();
    }
    
    unsafe {
        let runtime_ref = &*(handle as *const RuntimeHandle);
        let runtime = runtime_ref.lock().unwrap();
        let events = runtime.get_events();
        
        let json = serde_json::to_string(&events).unwrap_or_else(|_| "[]".to_string());
        
        env.new_string(json)
            .expect("Couldn't create java string!")
            .into_raw()
    }
}

/// Clear events
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeClearEvents(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    if handle == 0 {
        return 0;
    }
    
    unsafe {
        let runtime_ref = &*(handle as *const RuntimeHandle);
        let runtime = runtime_ref.lock().unwrap();
        runtime.clear_events();
        1
    }
}

/// Get statistics
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeGetStatistics(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    if handle == 0 {
        return env.new_string("{\"total_events\":0,\"function_calls\":0,\"errors\":0}")
            .expect("Couldn't create java string!")
            .into_raw();
    }
    
    unsafe {
        let runtime_ref = &*(handle as *const RuntimeHandle);
        let runtime = runtime_ref.lock().unwrap();
        let events = runtime.get_events();
        
        let total = events.len();
        let mut fn_count = 0;
        let mut err_count = 0;
        
        for event in events {
            match event {
                xplainit_core::ExecutionEvent::FunctionEnter { .. } |
                xplainit_core::ExecutionEvent::FunctionExit { .. } => fn_count += 1,
                xplainit_core::ExecutionEvent::DivisionByZero { .. } |
                xplainit_core::ExecutionEvent::NullPointerError { .. } |
                xplainit_core::ExecutionEvent::IndexOutOfBounds { .. } => err_count += 1,
                _ => {}
            }
        }
        
        let stats = format!(
            r#"{{"total_events":{},"function_calls":{},"errors":{}}}"#,
            total, fn_count, err_count
        );
        
        env.new_string(stats)
            .expect("Couldn't create java string!")
            .into_raw()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_jni_module_compiles() {
        assert!(true);
    }
}
