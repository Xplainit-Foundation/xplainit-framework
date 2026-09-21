/*
 * Xplainit -finstrument-functions bridge.
 *
 * Link this shim into a program compiled with GCC/Clang
 * `-finstrument-functions` to automatically forward every function entry and
 * exit to the Xplainit C FFI. The compiler-inserted callbacks
 * (__cyg_profile_func_enter / __cyg_profile_func_exit) receive function
 * ADDRESSES, so this shim resolves each address to a symbol name using
 * dladdr() (requires linking with -ldl and compiling with -rdynamic) before
 * forwarding to xplainit_on_function_enter / xplainit_on_function_exit.
 *
 * Every function in this shim is marked __attribute__((no_instrument_function))
 * so the shim never instruments itself (which would cause infinite recursion).
 */

#ifndef XPLAINIT_TRACE_H
#define XPLAINIT_TRACE_H

#ifdef __cplusplus
extern "C" {
#endif

/*
 * Initialize the global Xplainit handle used by the instrumentation shim.
 * Must be called (typically from main) before instrumented functions run.
 * Returns 1 on success, 0 on failure.
 */
int xplainit_instrument_init(void) __attribute__((no_instrument_function));

/*
 * Return the captured events as a JSON string. The caller must free the
 * returned pointer with xplainit_free_string(). Returns NULL if the shim was
 * not initialized.
 */
char *xplainit_instrument_get_events(void) __attribute__((no_instrument_function));

/*
 * Populate statistics for the captured events. Any output pointer may be NULL.
 * Returns 1 on success, 0 if the shim was not initialized.
 */
int xplainit_instrument_get_statistics(unsigned long *total_events,
                                       unsigned long *function_calls,
                                       unsigned long *errors)
    __attribute__((no_instrument_function));

/*
 * Report an exception/error from user code at the given location.
 * Returns 1 if recorded, 0 otherwise.
 */
int xplainit_instrument_report_exception(const char *error_type,
                                         const char *message,
                                         const char *file,
                                         unsigned int line)
    __attribute__((no_instrument_function));

/*
 * Tear down the global handle. Safe to call multiple times.
 */
void xplainit_instrument_shutdown(void) __attribute__((no_instrument_function));

#ifdef __cplusplus
}
#endif

#endif /* XPLAINIT_TRACE_H */
