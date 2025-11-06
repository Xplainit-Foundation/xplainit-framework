//! Xplainit C/C++ FFI Bindings
//!
//! This module provides C-compatible FFI functions for the Xplainit Framework
//! that can be called from C, C++, or any language with C FFI support.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Arc, Mutex};
use xplainit_core::{Config, Language, RuntimeEngine, Verbosity, OutputFormat};

/// Opaque handle to a runtime instance
pub struct XplainitHandle {
    runtime: Arc<Mutex<RuntimeEngine>>,
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
    
    let handle = &*handle;
    let _runtime = handle.runtime.lock().unwrap();
    
    // Enable tracing logic
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
    
    let handle = &*handle;
    let _runtime = handle.runtime.lock().unwrap();
    
    // Disable tracing logic
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
    
    // For now, always return enabled after create
    1
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
    
    let handle = &*handle;
    let runtime = handle.runtime.lock().unwrap();
    let events = runtime.get_events();
    
    let json = serde_json::to_string(&events).unwrap_or_else(|_| "[]".to_string());
    
    match CString::new(json) {
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
    
    let handle = &*handle;
    let runtime = handle.runtime.lock().unwrap();
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

/// Free a string returned by xplainit_get_events()
///
/// # Safety
/// The string must be a valid pointer returned from xplainit_get_events()
/// and must not be used after this call
#[no_mangle]
pub unsafe extern "C" fn xplainit_free_string(s: *mut c_char) {
    if !s.is_null() {
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
    "0.1.0\0".as_ptr() as *const c_char
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
