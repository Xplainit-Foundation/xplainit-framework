//! Xplainit C/C++ FFI Bindings
//!
//! This module provides C-compatible FFI functions for the Xplainit Framework
//! that can be called from C, C++, or any language with C FFI support.

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use xplainit_core::{
    Config, ExecutionEvent, Language, OutputFormat, RuntimeEngine, SourceLocation, Verbosity,
};

/// Opaque handle to a runtime instance
pub struct XplainitHandle {
    runtime: Arc<Mutex<RuntimeEngine>>,
    /// Whether recording is currently enabled. Recording FFI functions
    /// are gated on this flag so disabled handles capture nothing.
    enabled: AtomicBool,
}

/// Convert a C string pointer into an owned `String`, returning `None` for
/// null pointers or non-UTF-8 content.
///
/// # Safety
/// `ptr` must either be null or point to a valid null-terminated C string.
unsafe fn cstr_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: `ptr` was checked non-null immediately above. The function
    // contract requires the caller to pass either null or a valid,
    // null-terminated C string, so constructing a `CStr` from it is sound.
    // Non-UTF-8 content is handled by `to_str().ok()` returning `None` rather
    // than being assumed valid.
    CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_string())
}

/// Create a new Xplainit runtime instance
///
/// # Returns
/// Pointer to XplainitHandle on success, null on failure
///
/// # Safety
/// The returned pointer must be freed with xplainit_free()
#[no_mangle]
pub extern "C" fn xplainit_create() -> *mut XplainitHandle {
    let config = Config::new(Language::C)
        .with_verbosity(Verbosity::Normal)
        .with_output_format(OutputFormat::Json);

    let runtime = RuntimeEngine::new(config);

    let handle = Box::new(XplainitHandle {
        runtime: Arc::new(Mutex::new(runtime)),
        enabled: AtomicBool::new(true),
    });

    Box::into_raw(handle)
}

/// Free a Xplainit runtime instance
///
/// # Safety
/// The handle must be a valid pointer returned from xplainit_create()
/// and must not be used after this call
#[no_mangle]
pub unsafe extern "C" fn xplainit_free(handle: *mut XplainitHandle) {
    if !handle.is_null() {
        // SAFETY: `handle` is non-null (checked above) and, per contract, was
        // produced by `Box::into_raw` in `xplainit_create` and not yet freed.
        // Reconstructing the `Box` here takes ownership and drops it exactly
        // once; callers must not use the pointer afterwards.
        let _ = Box::from_raw(handle);
    }
}

/// Enable tracing for a runtime instance
///
/// # Arguments
/// * `handle` - Valid XplainitHandle pointer
///
/// # Returns
/// 1 on success, 0 on failure
///
/// # Safety
/// The handle must be a valid pointer returned from xplainit_create()
#[no_mangle]
pub unsafe extern "C" fn xplainit_enable(handle: *mut XplainitHandle) -> i32 {
    if handle.is_null() {
        return 0;
    }

    // SAFETY: `handle` is non-null (checked above) and, per contract, points to
    // a live `XplainitHandle` from `xplainit_create`. We only take a shared
    // reference and touch the atomic `enabled` flag, so no aliasing rules are
    // violated even under concurrent FFI calls.
    let handle = &*handle;
    handle.enabled.store(true, Ordering::SeqCst);
    1
}

/// Disable tracing for a runtime instance
///
/// # Arguments
/// * `handle` - Valid XplainitHandle pointer
///
/// # Returns
/// 1 on success, 0 on failure
///
/// # Safety
/// The handle must be a valid pointer returned from xplainit_create()
#[no_mangle]
pub unsafe extern "C" fn xplainit_disable(handle: *mut XplainitHandle) -> i32 {
    if handle.is_null() {
        return 0;
    }

    // SAFETY: `handle` is non-null (checked above) and, per contract, points to
    // a live `XplainitHandle` from `xplainit_create`. Only a shared reference to
    // the atomic `enabled` flag is taken, which is sound under concurrent calls.
    let handle = &*handle;
    handle.enabled.store(false, Ordering::SeqCst);
    1
}

/// Check if tracing is enabled
///
/// # Arguments
/// * `handle` - Valid XplainitHandle pointer
///
/// # Returns
/// 1 if enabled, 0 if disabled or invalid handle
///
/// # Safety
/// The handle must be a valid pointer returned from xplainit_create()
#[no_mangle]
pub unsafe extern "C" fn xplainit_is_enabled(handle: *mut XplainitHandle) -> i32 {
    if handle.is_null() {
        return 0;
    }

    // SAFETY: `handle` is non-null (checked above) and, per contract, points to
    // a live `XplainitHandle`. Only the atomic `enabled` flag is read through a
    // shared reference, which is sound.
    let handle = &*handle;
    if handle.enabled.load(Ordering::SeqCst) {
        1
    } else {
        0
    }
}

/// Get captured events as JSON string
///
/// # Arguments
/// * `handle` - Valid XplainitHandle pointer
///
/// # Returns
/// Pointer to null-terminated JSON string on success, null on failure
/// The returned string must be freed with xplainit_free_string()
///
/// # Safety
/// The handle must be a valid pointer returned from xplainit_create()
#[no_mangle]
pub unsafe extern "C" fn xplainit_get_events(handle: *mut XplainitHandle) -> *mut c_char {
    if handle.is_null() {
        return ptr::null_mut();
    }

    // SAFETY: `handle` is non-null (checked above) and, per contract, points to
    // a live `XplainitHandle`. A shared reference is taken and the `Mutex`
    // guards concurrent access to the runtime.
    let handle = &*handle;
    let runtime = handle.runtime.lock().unwrap();
    let events = runtime.get_events();

    let json = serde_json::to_string(&events).unwrap_or_else(|_| "[]".to_string());

    match CString::new(json) {
        // Ownership of the buffer is transferred to the caller, who must free it
        // with `xplainit_free_string`.
        Ok(c_str) => c_str.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Clear all captured events
///
/// # Arguments
/// * `handle` - Valid XplainitHandle pointer
///
/// # Returns
/// 1 on success, 0 on failure
///
/// # Safety
/// The handle must be a valid pointer returned from xplainit_create()
#[no_mangle]
pub unsafe extern "C" fn xplainit_clear_events(handle: *mut XplainitHandle) -> i32 {
    if handle.is_null() {
        return 0;
    }

    // SAFETY: `handle` is non-null (checked above) and, per contract, points to
    // a live `XplainitHandle`. A shared reference is taken and the `Mutex`
    // guards concurrent access to the runtime.
    let handle = &*handle;
    let runtime = handle.runtime.lock().unwrap();
    runtime.clear_events();

    1
}

/// Get statistics about captured events
///
/// # Arguments
/// * `handle` - Valid XplainitHandle pointer
/// * `total_events` - Output pointer for total event count (can be null)
/// * `function_calls` - Output pointer for function call count (can be null)
/// * `errors` - Output pointer for error count (can be null)
///
/// # Returns
/// 1 on success, 0 on failure
///
/// # Safety
/// The handle must be a valid pointer returned from xplainit_create()
/// The output pointers, if not null, must be valid
#[no_mangle]
pub unsafe extern "C" fn xplainit_get_statistics(
    handle: *mut XplainitHandle,
    total_events: *mut usize,
    function_calls: *mut usize,
    errors: *mut usize,
) -> i32 {
    if handle.is_null() {
        return 0;
    }

    // SAFETY: `handle` is non-null (checked above) and, per contract, points to
    // a live `XplainitHandle`. A shared reference is taken and the `Mutex`
    // guards concurrent access to the runtime.
    let handle = &*handle;
    let runtime = handle.runtime.lock().unwrap();
    let events = runtime.get_events();

    let total = events.len();
    let mut fn_count = 0;
    let mut err_count = 0;

    for event in events {
        match event {
            xplainit_core::ExecutionEvent::FunctionEnter { .. }
            | xplainit_core::ExecutionEvent::FunctionExit { .. } => fn_count += 1,
            xplainit_core::ExecutionEvent::DivisionByZero { .. }
            | xplainit_core::ExecutionEvent::NullPointerError { .. }
            | xplainit_core::ExecutionEvent::IndexOutOfBounds { .. } => err_count += 1,
            _ => {}
        }
    }

    // SAFETY: each output pointer is written only after an explicit null check,
    // so we never dereference null. Per contract, any non-null output pointer
    // points to a writable `usize` supplied by the caller.
    if !total_events.is_null() {
        *total_events = total;
    }
    if !function_calls.is_null() {
        *function_calls = fn_count;
    }
    if !errors.is_null() {
        *errors = err_count;
    }

    1
}

/// Record a function entry event
///
/// # Arguments
/// * `handle` - Valid XplainitHandle pointer
/// * `name` - Function name (null-terminated C string, may be null)
/// * `file` - Source file (null-terminated C string, may be null)
/// * `line` - Source line number
///
/// # Returns
/// 1 if the event was recorded, 0 otherwise (null handle or tracing disabled)
///
/// # Safety
/// The handle must be a valid pointer returned from xplainit_create().
/// `name` and `file`, if not null, must point to valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn xplainit_on_function_enter(
    handle: *mut XplainitHandle,
    name: *const c_char,
    file: *const c_char,
    line: u32,
) -> i32 {
    if handle.is_null() {
        return 0;
    }

    // SAFETY: `handle` is non-null (checked above) and, per contract, points to
    // a live `XplainitHandle`. A shared reference is taken; the `Mutex` guards
    // the runtime. The `name`/`file` pointers are only dereferenced inside
    // `cstr_to_string`, which null-checks them.
    let handle = &*handle;
    if !handle.enabled.load(Ordering::SeqCst) {
        return 0;
    }

    let name = cstr_to_string(name).unwrap_or_else(|| "<unknown>".to_string());
    let file = cstr_to_string(file).unwrap_or_else(|| "<unknown>".to_string());

    let event = ExecutionEvent::FunctionEnter {
        id: uuid::Uuid::new_v4(),
        name,
        args: HashMap::new(),
        location: SourceLocation::new(file, line as usize, 0),
        timestamp: chrono::Utc::now(),
    };

    let runtime = handle.runtime.lock().unwrap();
    runtime.event_store().record(event);

    1
}

/// Record a function exit event
///
/// # Arguments
/// * `handle` - Valid XplainitHandle pointer
/// * `name` - Function name (null-terminated C string, may be null)
/// * `file` - Source file (null-terminated C string, may be null)
/// * `line` - Source line number
///
/// # Returns
/// 1 if the event was recorded, 0 otherwise (null handle or tracing disabled)
///
/// # Safety
/// The handle must be a valid pointer returned from xplainit_create().
/// `name` and `file`, if not null, must point to valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn xplainit_on_function_exit(
    handle: *mut XplainitHandle,
    name: *const c_char,
    file: *const c_char,
    line: u32,
) -> i32 {
    if handle.is_null() {
        return 0;
    }

    // SAFETY: `handle` is non-null (checked above) and, per contract, points to
    // a live `XplainitHandle`. A shared reference is taken; the `Mutex` guards
    // the runtime. The `name`/`file` pointers are only dereferenced inside
    // `cstr_to_string`, which null-checks them.
    let handle = &*handle;
    if !handle.enabled.load(Ordering::SeqCst) {
        return 0;
    }

    let name = cstr_to_string(name).unwrap_or_else(|| "<unknown>".to_string());
    // file/line are part of the FFI signature for symmetry with enter/exception
    // and future use; FunctionExit does not carry a location field.
    let _ = (cstr_to_string(file), line);

    let event = ExecutionEvent::FunctionExit {
        id: uuid::Uuid::new_v4(),
        name,
        return_value: None,
        duration: Duration::from_secs(0),
        timestamp: chrono::Utc::now(),
    };

    let runtime = handle.runtime.lock().unwrap();
    runtime.event_store().record(event);

    1
}

/// Record an exception event
///
/// # Arguments
/// * `handle` - Valid XplainitHandle pointer
/// * `error_type` - Exception type (null-terminated C string, may be null)
/// * `message` - Exception message (null-terminated C string, may be null)
/// * `file` - Source file (null-terminated C string, may be null)
/// * `line` - Source line number
///
/// # Returns
/// 1 if the event was recorded, 0 otherwise (null handle or tracing disabled)
///
/// # Safety
/// The handle must be a valid pointer returned from xplainit_create().
/// The string arguments, if not null, must point to valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn xplainit_on_exception(
    handle: *mut XplainitHandle,
    error_type: *const c_char,
    message: *const c_char,
    file: *const c_char,
    line: u32,
) -> i32 {
    if handle.is_null() {
        return 0;
    }

    // SAFETY: `handle` is non-null (checked above) and, per contract, points to
    // a live `XplainitHandle`. A shared reference is taken; the `Mutex` guards
    // the runtime. The string pointers are only dereferenced inside
    // `cstr_to_string`, which null-checks them.
    let handle = &*handle;
    if !handle.enabled.load(Ordering::SeqCst) {
        return 0;
    }

    let error_type = cstr_to_string(error_type).unwrap_or_else(|| "Error".to_string());
    let message = cstr_to_string(message).unwrap_or_default();
    let file = cstr_to_string(file).unwrap_or_else(|| "<unknown>".to_string());

    let event = ExecutionEvent::Exception {
        id: uuid::Uuid::new_v4(),
        error_type,
        message,
        location: SourceLocation::new(file, line as usize, 0),
        stack_trace: vec![],
        caught: false,
        timestamp: chrono::Utc::now(),
    };

    let runtime = handle.runtime.lock().unwrap();
    runtime.event_store().record(event);

    1
}

/// Free a string returned by xplainit_get_events()
///
/// # Safety
/// The string must be a valid pointer returned from xplainit_get_events()
/// and must not be used after this call
#[no_mangle]
pub unsafe extern "C" fn xplainit_free_string(s: *mut c_char) {
    if !s.is_null() {
        // SAFETY: `s` is non-null (checked above) and, per contract, was
        // produced by `CString::into_raw` in `xplainit_get_events` and not yet
        // freed. Reconstructing the `CString` reclaims and drops it exactly
        // once; the caller must not use the pointer afterwards.
        let _ = CString::from_raw(s);
    }
}

/// Get version string
///
/// # Returns
/// Pointer to static null-terminated version string
///
/// # Safety
/// This function is always safe to call
#[no_mangle]
pub extern "C" fn xplainit_version() -> *const c_char {
    c"0.1.0".as_ptr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn test_create_and_free() {
        unsafe {
            let handle = xplainit_create();
            assert!(!handle.is_null());
            xplainit_free(handle);
        }
    }

    #[test]
    fn test_enable_disable() {
        unsafe {
            let handle = xplainit_create();
            assert_eq!(xplainit_enable(handle), 1);
            assert_eq!(xplainit_is_enabled(handle), 1);
            assert_eq!(xplainit_disable(handle), 1);
            xplainit_free(handle);
        }
    }

    #[test]
    fn test_get_events() {
        unsafe {
            let handle = xplainit_create();
            let events = xplainit_get_events(handle);
            assert!(!events.is_null());
            xplainit_free_string(events);
            xplainit_free(handle);
        }
    }

    #[test]
    fn test_statistics() {
        unsafe {
            let handle = xplainit_create();
            let mut total: usize = 0;
            let mut functions: usize = 0;
            let mut errors: usize = 0;

            let result = xplainit_get_statistics(
                handle,
                &mut total as *mut usize,
                &mut functions as *mut usize,
                &mut errors as *mut usize,
            );

            assert_eq!(result, 1);
            assert_eq!(total, 0);

            xplainit_free(handle);
        }
    }

    #[test]
    fn test_version() {
        unsafe {
            let version = xplainit_version();
            assert!(!version.is_null());
            let c_str = CStr::from_ptr(version);
            assert_eq!(c_str.to_str().unwrap(), "0.1.0");
        }
    }

    #[test]
    fn test_enable_disable_flag_gates_state() {
        unsafe {
            let handle = xplainit_create();
            // Created enabled by default.
            assert_eq!(xplainit_is_enabled(handle), 1);
            assert_eq!(xplainit_disable(handle), 1);
            assert_eq!(xplainit_is_enabled(handle), 0);
            assert_eq!(xplainit_enable(handle), 1);
            assert_eq!(xplainit_is_enabled(handle), 1);
            xplainit_free(handle);
        }
    }

    #[test]
    fn test_recording_when_enabled() {
        unsafe {
            let handle = xplainit_create();
            let name = CString::new("compute").unwrap();
            let file = CString::new("demo.c").unwrap();

            assert_eq!(
                xplainit_on_function_enter(handle, name.as_ptr(), file.as_ptr(), 10),
                1
            );
            assert_eq!(
                xplainit_on_function_exit(handle, name.as_ptr(), file.as_ptr(), 12),
                1
            );

            let mut total: usize = 0;
            let mut functions: usize = 0;
            let mut errors: usize = 0;
            // NOTE: get_statistics drains the store via get_events, so read it
            // before get_events. We check statistics here first.
            let result = xplainit_get_statistics(
                handle,
                &mut total as *mut usize,
                &mut functions as *mut usize,
                &mut errors as *mut usize,
            );
            assert_eq!(result, 1);
            assert_eq!(total, 2);
            assert_eq!(functions, 2);
            assert_eq!(errors, 0);

            xplainit_free(handle);
        }
    }

    #[test]
    fn test_recording_events_json_reflects_names() {
        unsafe {
            let handle = xplainit_create();
            let name = CString::new("factorial").unwrap();
            let file = CString::new("demo.c").unwrap();
            xplainit_on_function_enter(handle, name.as_ptr(), file.as_ptr(), 5);
            xplainit_on_function_exit(handle, name.as_ptr(), file.as_ptr(), 7);

            let events_ptr = xplainit_get_events(handle);
            assert!(!events_ptr.is_null());
            let json = CStr::from_ptr(events_ptr).to_str().unwrap().to_string();
            assert!(json.contains("factorial"));
            assert!(json.contains("function_enter") || json.contains("FunctionEnter"));
            xplainit_free_string(events_ptr);
            xplainit_free(handle);
        }
    }

    #[test]
    fn test_exception_recording() {
        unsafe {
            let handle = xplainit_create();
            let etype = CString::new("RuntimeError").unwrap();
            let msg = CString::new("boom").unwrap();
            let file = CString::new("demo.c").unwrap();
            assert_eq!(
                xplainit_on_exception(handle, etype.as_ptr(), msg.as_ptr(), file.as_ptr(), 42),
                1
            );

            let events_ptr = xplainit_get_events(handle);
            let json = CStr::from_ptr(events_ptr).to_str().unwrap().to_string();
            assert!(json.contains("RuntimeError"));
            assert!(json.contains("boom"));
            xplainit_free_string(events_ptr);
            xplainit_free(handle);
        }
    }

    #[test]
    fn test_nothing_recorded_when_disabled() {
        unsafe {
            let handle = xplainit_create();
            assert_eq!(xplainit_disable(handle), 1);

            let name = CString::new("compute").unwrap();
            let file = CString::new("demo.c").unwrap();
            // All recording calls should return 0 (not recorded) while disabled.
            assert_eq!(
                xplainit_on_function_enter(handle, name.as_ptr(), file.as_ptr(), 1),
                0
            );
            assert_eq!(
                xplainit_on_function_exit(handle, name.as_ptr(), file.as_ptr(), 2),
                0
            );
            let etype = CString::new("Error").unwrap();
            let msg = CString::new("nope").unwrap();
            assert_eq!(
                xplainit_on_exception(handle, etype.as_ptr(), msg.as_ptr(), file.as_ptr(), 3),
                0
            );

            let mut total: usize = 0;
            xplainit_get_statistics(
                handle,
                &mut total as *mut usize,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            assert_eq!(total, 0);

            xplainit_free(handle);
        }
    }

    #[test]
    fn test_null_handle_statistics_returns_safe_error() {
        unsafe {
            // A null handle must take the guarded error path (return 0) instead
            // of dereferencing the pointer. Out-pointers are also null here to
            // confirm the null-checked writes never fire.
            let result = xplainit_get_statistics(
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
            assert_eq!(result, 0);

            // Non-null handle with null out-pointers must still be safe.
            let handle = xplainit_create();
            let result =
                xplainit_get_statistics(handle, ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
            assert_eq!(result, 1);
            xplainit_free(handle);
        }
    }

    #[test]
    fn test_null_handle_enable_disable_returns_safe_error() {
        unsafe {
            assert_eq!(xplainit_enable(ptr::null_mut()), 0);
            assert_eq!(xplainit_disable(ptr::null_mut()), 0);
            assert_eq!(xplainit_is_enabled(ptr::null_mut()), 0);
            assert_eq!(xplainit_clear_events(ptr::null_mut()), 0);
            assert!(xplainit_get_events(ptr::null_mut()).is_null());
        }
    }

    #[test]
    fn test_null_handle_recording_is_safe() {
        unsafe {
            let name = CString::new("x").unwrap();
            let file = CString::new("f").unwrap();
            assert_eq!(
                xplainit_on_function_enter(ptr::null_mut(), name.as_ptr(), file.as_ptr(), 1),
                0
            );
            assert_eq!(
                xplainit_on_function_exit(ptr::null_mut(), name.as_ptr(), file.as_ptr(), 1),
                0
            );
            assert_eq!(
                xplainit_on_exception(
                    ptr::null_mut(),
                    name.as_ptr(),
                    name.as_ptr(),
                    file.as_ptr(),
                    1
                ),
                0
            );
        }
    }
}
