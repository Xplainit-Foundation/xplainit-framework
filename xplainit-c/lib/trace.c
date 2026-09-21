/*
 * Xplainit -finstrument-functions bridge implementation.
 *
 * See trace.h for usage. Compile this file (and everything that should be
 * traced) with -finstrument-functions, and link with -rdynamic -ldl plus the
 * Xplainit C FFI library (-lxplainit_c). Every function here is marked
 * no_instrument_function so the shim does not recurse into itself.
 */

#define _GNU_SOURCE
#include "trace.h"
#include "xplainit-c.h"

#include <dlfcn.h>
#include <stddef.h>

/* Global handle shared by the compiler-inserted callbacks. */
static struct XplainitHandle *g_handle = NULL;

int xplainit_instrument_init(void) {
    if (g_handle == NULL) {
        g_handle = xplainit_create();
    }
    if (g_handle == NULL) {
        return 0;
    }
    xplainit_enable(g_handle);
    return 1;
}

char *xplainit_instrument_get_events(void) {
    if (g_handle == NULL) {
        return NULL;
    }
    return xplainit_get_events(g_handle);
}

int xplainit_instrument_get_statistics(unsigned long *total_events,
                                       unsigned long *function_calls,
                                       unsigned long *errors) {
    if (g_handle == NULL) {
        return 0;
    }
    size_t total = 0, functions = 0, errs = 0;
    int rc = xplainit_get_statistics(g_handle, &total, &functions, &errs);
    if (total_events) {
        *total_events = (unsigned long)total;
    }
    if (function_calls) {
        *function_calls = (unsigned long)functions;
    }
    if (errors) {
        *errors = (unsigned long)errs;
    }
    return rc;
}

int xplainit_instrument_report_exception(const char *error_type,
                                         const char *message,
                                         const char *file,
                                         unsigned int line) {
    if (g_handle == NULL) {
        return 0;
    }
    return xplainit_on_exception(g_handle, error_type, message, file, line);
}

void xplainit_instrument_shutdown(void) {
    if (g_handle != NULL) {
        xplainit_free(g_handle);
        g_handle = NULL;
    }
}

/*
 * Resolve a function address to a symbol name via dladdr(). Falls back to
 * "<unknown>" if the symbol cannot be resolved. dladdr requires -rdynamic (or
 * exported symbols) and linking with -ldl.
 */
static const char *resolve_symbol(void *func)
    __attribute__((no_instrument_function));

static const char *resolve_symbol(void *func) {
    Dl_info info;
    if (dladdr(func, &info) != 0 && info.dli_sname != NULL) {
        return info.dli_sname;
    }
    return "<unknown>";
}

/*
 * GCC/Clang instrumentation callbacks. These fire on entry/exit of every
 * function compiled with -finstrument-functions in the linked objects.
 */
void __cyg_profile_func_enter(void *this_fn, void *call_site)
    __attribute__((no_instrument_function));
void __cyg_profile_func_exit(void *this_fn, void *call_site)
    __attribute__((no_instrument_function));

void __cyg_profile_func_enter(void *this_fn, void *call_site) {
    (void)call_site;
    if (g_handle == NULL) {
        return;
    }
    const char *name = resolve_symbol(this_fn);
    xplainit_on_function_enter(g_handle, name, "<c>", 0);
}

void __cyg_profile_func_exit(void *this_fn, void *call_site) {
    (void)call_site;
    if (g_handle == NULL) {
        return;
    }
    const char *name = resolve_symbol(this_fn);
    xplainit_on_function_exit(g_handle, name, "<c>", 0);
}
