//! Xplainit Java JNI Bindings
//!
//! This module provides JNI (Java Native Interface) bindings for the Xplainit Framework

use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jlong, jstring};
use jni::JNIEnv;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use xplainit_core::{
    Config, ExecutionEvent, Language, OutputFormat, RuntimeEngine, SourceLocation, Verbosity,
};

/// Runtime instance stored in a Java object field as an opaque `jlong` handle.
///
/// The handle wraps both the engine and a real `enabled` flag so the recording
/// natives can be gated (mirroring the C FFI `XplainitHandle`).
struct JavaRuntime {
    runtime: Arc<Mutex<RuntimeEngine>>,
    enabled: AtomicBool,
}

type RuntimeHandle = JavaRuntime;

/// Build a fresh Java-flavoured runtime engine.
fn new_engine() -> RuntimeEngine {
    let config = Config::new(Language::Java)
        .with_verbosity(Verbosity::Normal)
        .with_output_format(OutputFormat::Json);
    RuntimeEngine::new(config)
}

/// Build a `FunctionEnter` event from method metadata.
///
/// Factored out so the pure event-construction logic can be unit tested
/// without a live `JNIEnv`.
fn build_method_enter_event(name: &str, signature: &str, line: i32) -> ExecutionEvent {
    let file = if signature.is_empty() {
        "<jvm>".to_string()
    } else {
        signature.to_string()
    };
    ExecutionEvent::FunctionEnter {
        id: uuid::Uuid::new_v4(),
        name: name.to_string(),
        args: HashMap::new(),
        location: SourceLocation::new(file, line.max(0) as usize, 0),
        timestamp: chrono::Utc::now(),
    }
}

/// Build a `FunctionExit` event from method metadata.
fn build_method_exit_event(name: &str) -> ExecutionEvent {
    ExecutionEvent::FunctionExit {
        id: uuid::Uuid::new_v4(),
        name: name.to_string(),
        return_value: None,
        duration: Duration::from_secs(0),
        timestamp: chrono::Utc::now(),
    }
}

/// Build an `Exception` event from exception metadata.
fn build_exception_event(error_type: &str, message: &str, file: &str, line: i32) -> ExecutionEvent {
    ExecutionEvent::Exception {
        id: uuid::Uuid::new_v4(),
        error_type: error_type.to_string(),
        message: message.to_string(),
        location: SourceLocation::new(file.to_string(), line.max(0) as usize, 0),
        stack_trace: vec![],
        caught: false,
        timestamp: chrono::Utc::now(),
    }
}

/// Read a (possibly null) `JString` into an owned Rust `String`.
///
/// Returns `None` for a null reference so callers can substitute defaults.
fn jstring_to_string(env: &mut JNIEnv, s: &JString) -> Option<String> {
    if s.is_null() {
        return None;
    }
    env.get_string(s).ok().map(|js| js.into())
}

/// Process-wide runtime used by the JVMTI agent.
///
/// The agent lives in a separate shared object (`libxplainit_agent.so`) and
/// drives recording through the `nativeAgent*` statics on the `Xplainit` Java
/// class. Those statics funnel into this single global engine, which is created
/// lazily the first time the agent initialises.
static AGENT_RUNTIME: OnceLock<JavaRuntime> = OnceLock::new();

fn agent_runtime() -> &'static JavaRuntime {
    AGENT_RUNTIME.get_or_init(|| JavaRuntime {
        runtime: Arc::new(Mutex::new(new_engine())),
        enabled: AtomicBool::new(true),
    })
}

/// Create a new Xplainit runtime
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeCreate(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    let handle = Box::new(JavaRuntime {
        runtime: Arc::new(Mutex::new(new_engine())),
        enabled: AtomicBool::new(false),
    });

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
        // SAFETY: `handle` is non-zero (checked above) and, per contract, was
        // produced by `Box::into_raw` in `nativeCreate` and not yet freed.
        // Reconstructing the `Box` takes ownership and drops it exactly once;
        // Java must not use the handle afterwards.
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

    // SAFETY: `handle` is non-zero (checked above) and, per contract, is a live
    // `RuntimeHandle` pointer from `nativeCreate`. Only a shared reference to the
    // atomic `enabled` flag is taken, which is sound under concurrent JNI calls.
    unsafe {
        let runtime_ref = &*(handle as *const RuntimeHandle);
        runtime_ref.enabled.store(true, Ordering::SeqCst);
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

    // SAFETY: `handle` is non-zero (checked above) and, per contract, is a live
    // `RuntimeHandle` pointer. Only a shared reference to the atomic `enabled`
    // flag is taken, which is sound under concurrent JNI calls.
    unsafe {
        let runtime_ref = &*(handle as *const RuntimeHandle);
        runtime_ref.enabled.store(false, Ordering::SeqCst);
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

    // SAFETY: `handle` is non-zero (checked above) and, per contract, is a live
    // `RuntimeHandle` pointer. Only the atomic `enabled` flag is read through a
    // shared reference, which is sound.
    unsafe {
        let runtime_ref = &*(handle as *const RuntimeHandle);
        if runtime_ref.enabled.load(Ordering::SeqCst) {
            1
        } else {
            0
        }
    }
}

/// Get events as JSON string
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeGetEvents(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    if handle == 0 {
        return env
            .new_string("[]")
            .expect("Couldn't create java string!")
            .into_raw();
    }

    // SAFETY: `handle` is non-zero (checked above) and, per contract, is a live
    // `RuntimeHandle` pointer. A shared reference is taken and the `Mutex`
    // guards concurrent access to the runtime.
    unsafe {
        let runtime_ref = &*(handle as *const RuntimeHandle);
        let runtime = runtime_ref.runtime.lock().unwrap();
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

    // SAFETY: `handle` is non-zero (checked above) and, per contract, is a live
    // `RuntimeHandle` pointer. A shared reference is taken and the `Mutex`
    // guards concurrent access to the runtime.
    unsafe {
        let runtime_ref = &*(handle as *const RuntimeHandle);
        let runtime = runtime_ref.runtime.lock().unwrap();
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
        return env
            .new_string("{\"total_events\":0,\"function_calls\":0,\"errors\":0}")
            .expect("Couldn't create java string!")
            .into_raw();
    }

    // SAFETY: `handle` is non-zero (checked above) and, per contract, is a live
    // `RuntimeHandle` pointer. A shared reference is taken and the `Mutex`
    // guards concurrent access to the runtime.
    unsafe {
        let runtime_ref = &*(handle as *const RuntimeHandle);
        let runtime = runtime_ref.runtime.lock().unwrap();
        let events = runtime.get_events();

        let total = events.len();
        let mut fn_count = 0;
        let mut err_count = 0;

        for event in events {
            match event {
                xplainit_core::ExecutionEvent::FunctionEnter { .. }
                | xplainit_core::ExecutionEvent::FunctionExit { .. } => fn_count += 1,
                e if e.is_error() => err_count += 1,
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

// ===== Instance recording natives =====
//
// These drive recording through an explicit `handle` (the same one returned by
// nativeCreate). They let plain Java code push method events without the agent.

/// Record a method-entry event on the given handle.
///
/// Returns 1 if recorded, 0 if the handle is null or tracing is disabled.
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeOnMethodEnter(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    name: JString,
    signature: JString,
    line: jint,
) -> jboolean {
    if handle == 0 {
        return 0;
    }

    let name = jstring_to_string(&mut env, &name).unwrap_or_else(|| "<unknown>".to_string());
    let signature = jstring_to_string(&mut env, &signature).unwrap_or_default();

    // SAFETY: `handle` is non-zero (checked above) and, per contract, is a live
    // `RuntimeHandle` pointer. A shared reference is taken; the atomic flag and
    // `Mutex` make concurrent access sound. The `JString` args were already
    // converted to owned Rust strings (null-checked) before this block.
    unsafe {
        let runtime_ref = &*(handle as *const RuntimeHandle);
        if !runtime_ref.enabled.load(Ordering::SeqCst) {
            return 0;
        }
        let event = build_method_enter_event(&name, &signature, line);
        let runtime = runtime_ref.runtime.lock().unwrap();
        runtime.event_store().record(event);
        1
    }
}

/// Record a method-exit event on the given handle.
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeOnMethodExit(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    name: JString,
) -> jboolean {
    if handle == 0 {
        return 0;
    }

    let name = jstring_to_string(&mut env, &name).unwrap_or_else(|| "<unknown>".to_string());

    // SAFETY: `handle` is non-zero (checked above) and, per contract, is a live
    // `RuntimeHandle` pointer. A shared reference is taken; the atomic flag and
    // `Mutex` make concurrent access sound. The `JString` arg was already
    // converted to an owned Rust string (null-checked) before this block.
    unsafe {
        let runtime_ref = &*(handle as *const RuntimeHandle);
        if !runtime_ref.enabled.load(Ordering::SeqCst) {
            return 0;
        }
        let event = build_method_exit_event(&name);
        let runtime = runtime_ref.runtime.lock().unwrap();
        runtime.event_store().record(event);
        1
    }
}

/// Record an exception event on the given handle.
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeOnException(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    error_type: JString,
    message: JString,
    file: JString,
    line: jint,
) -> jboolean {
    if handle == 0 {
        return 0;
    }

    let error_type =
        jstring_to_string(&mut env, &error_type).unwrap_or_else(|| "Exception".to_string());
    let message = jstring_to_string(&mut env, &message).unwrap_or_default();
    let file = jstring_to_string(&mut env, &file).unwrap_or_else(|| "<jvm>".to_string());

    // SAFETY: `handle` is non-zero (checked above) and, per contract, is a live
    // `RuntimeHandle` pointer. A shared reference is taken; the atomic flag and
    // `Mutex` make concurrent access sound. The `JString` args were already
    // converted to owned Rust strings (null-checked) before this block.
    unsafe {
        let runtime_ref = &*(handle as *const RuntimeHandle);
        if !runtime_ref.enabled.load(Ordering::SeqCst) {
            return 0;
        }
        let event = build_exception_event(&error_type, &message, &file, line);
        let runtime = runtime_ref.runtime.lock().unwrap();
        runtime.event_store().record(event);
        1
    }
}

// ===== Agent recording natives =====
//
// The JVMTI agent (libxplainit_agent.so) records into a single process-wide
// engine via these statics, so it does not need to thread a handle around.

/// Initialise (and enable) the process-wide agent runtime.
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeAgentInit(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    let rt = agent_runtime();
    rt.enabled.store(true, Ordering::SeqCst);
    1
}

/// Record a method-entry event into the agent runtime.
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeAgentOnMethodEnter(
    mut env: JNIEnv,
    _class: JClass,
    name: JString,
    signature: JString,
    line: jint,
) -> jboolean {
    let rt = agent_runtime();
    if !rt.enabled.load(Ordering::SeqCst) {
        return 0;
    }
    let name = jstring_to_string(&mut env, &name).unwrap_or_else(|| "<unknown>".to_string());
    let signature = jstring_to_string(&mut env, &signature).unwrap_or_default();
    let event = build_method_enter_event(&name, &signature, line);
    let runtime = rt.runtime.lock().unwrap();
    runtime.event_store().record(event);
    1
}

/// Record a method-exit event into the agent runtime.
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeAgentOnMethodExit(
    mut env: JNIEnv,
    _class: JClass,
    name: JString,
) -> jboolean {
    let rt = agent_runtime();
    if !rt.enabled.load(Ordering::SeqCst) {
        return 0;
    }
    let name = jstring_to_string(&mut env, &name).unwrap_or_else(|| "<unknown>".to_string());
    let event = build_method_exit_event(&name);
    let runtime = rt.runtime.lock().unwrap();
    runtime.event_store().record(event);
    1
}

/// Record an exception event into the agent runtime.
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeAgentOnException(
    mut env: JNIEnv,
    _class: JClass,
    error_type: JString,
    message: JString,
    file: JString,
    line: jint,
) -> jboolean {
    let rt = agent_runtime();
    if !rt.enabled.load(Ordering::SeqCst) {
        return 0;
    }
    let error_type =
        jstring_to_string(&mut env, &error_type).unwrap_or_else(|| "Exception".to_string());
    let message = jstring_to_string(&mut env, &message).unwrap_or_default();
    let file = jstring_to_string(&mut env, &file).unwrap_or_else(|| "<jvm>".to_string());
    let event = build_exception_event(&error_type, &message, &file, line);
    let runtime = rt.runtime.lock().unwrap();
    runtime.event_store().record(event);
    1
}

/// Get all events captured by the agent runtime as a JSON string (drains store).
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeAgentGetEvents(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let rt = agent_runtime();
    let runtime = rt.runtime.lock().unwrap();
    let events = runtime.get_events();
    let json = serde_json::to_string(&events).unwrap_or_else(|_| "[]".to_string());
    env.new_string(json)
        .expect("Couldn't create java string!")
        .into_raw()
}

/// Get statistics for the agent runtime as a JSON string (drains store).
#[no_mangle]
pub extern "system" fn Java_io_xplainit_Xplainit_nativeAgentGetStatistics(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let rt = agent_runtime();
    let runtime = rt.runtime.lock().unwrap();
    let events = runtime.get_events();

    let total = events.len();
    let mut fn_count = 0;
    let mut err_count = 0;
    for event in events {
        match event {
            ExecutionEvent::FunctionEnter { .. } | ExecutionEvent::FunctionExit { .. } => {
                fn_count += 1
            }
            e if e.is_error() => err_count += 1,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jni_module_compiles() {
        // This test just ensures the JNI module compiles successfully
    }

    #[test]
    fn test_build_method_enter_event() {
        let event = build_method_enter_event("fibonacci", "io/xplainit/examples/BasicExample", 42);
        match event {
            ExecutionEvent::FunctionEnter { name, location, .. } => {
                assert_eq!(name, "fibonacci");
                assert_eq!(location.file, "io/xplainit/examples/BasicExample");
                assert_eq!(location.line, 42);
            }
            _ => panic!("expected FunctionEnter"),
        }
    }

    #[test]
    fn test_build_method_enter_event_empty_signature() {
        let event = build_method_enter_event("m", "", -1);
        match event {
            ExecutionEvent::FunctionEnter { location, .. } => {
                assert_eq!(location.file, "<jvm>");
                // Negative line clamps to 0.
                assert_eq!(location.line, 0);
            }
            _ => panic!("expected FunctionEnter"),
        }
    }

    #[test]
    fn test_build_method_exit_event() {
        let event = build_method_exit_event("compute");
        match event {
            ExecutionEvent::FunctionExit { name, .. } => assert_eq!(name, "compute"),
            _ => panic!("expected FunctionExit"),
        }
    }

    #[test]
    fn test_build_exception_event() {
        let event =
            build_exception_event("ArithmeticException", "/ by zero", "BasicExample.java", 10);
        match event {
            ExecutionEvent::Exception {
                error_type,
                message,
                location,
                caught,
                ..
            } => {
                assert_eq!(error_type, "ArithmeticException");
                assert_eq!(message, "/ by zero");
                assert_eq!(location.line, 10);
                assert!(!caught);
            }
            _ => panic!("expected Exception"),
        }
    }

    #[test]
    fn test_agent_runtime_records_and_drains() {
        let rt = agent_runtime();
        rt.enabled.store(true, Ordering::SeqCst);
        {
            let runtime = rt.runtime.lock().unwrap();
            runtime
                .event_store()
                .record(build_method_enter_event("agentFn", "Sig", 1));
            runtime
                .event_store()
                .record(build_method_exit_event("agentFn"));
        }
        let runtime = rt.runtime.lock().unwrap();
        let events = runtime.get_events();
        assert!(events.len() >= 2);
        // Draining removes them.
        assert_eq!(runtime.get_events().len(), 0);
    }
}
