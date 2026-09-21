/**
 * Example: C Program with Automatic Tracing via -finstrument-functions
 *
 * Every function in this file is compiled with -finstrument-functions, so
 * GCC/Clang inserts calls to __cyg_profile_func_enter/exit (implemented in
 * ../lib/trace.c) which resolve the function address to a name via dladdr()
 * and forward it to the Xplainit C FFI recording functions.
 *
 * Build and run via examples/build.sh (which also asserts real captured
 * events). See that script for the exact compile/link flags.
 */

#include <stdio.h>
#include <stdlib.h>

#include "trace.h"
#include "xplainit-c.h"

/* Simple function to trace. */
int add(int a, int b) {
    return a + b;
}

int multiply(int x, int y) {
    return x * y;
}

/* Recursive function. */
int factorial(int n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

/* Nested function calls. */
int calculate_something(int a, int b, int c) {
    int sum = add(a, b);
    int product = multiply(sum, c);
    return product;
}

/* Error path: reports an exception to the tracer when dividing by zero. */
int safe_divide(int numerator, int denominator) {
    if (denominator == 0) {
        xplainit_instrument_report_exception(
            "DivisionByZero", "attempted to divide by zero", __FILE__, __LINE__);
        return 0;
    }
    return numerator / denominator;
}

int main(void) {
    printf("=================================================================\n");
    printf("Xplainit C Automatic Tracing Example (-finstrument-functions)\n");
    printf("=================================================================\n\n");

    if (!xplainit_instrument_init()) {
        fprintf(stderr, "ERROR: failed to initialize Xplainit tracer\n");
        return 1;
    }

    printf("Test 1: add(5, 3)\n");
    printf("Result: %d\n\n", add(5, 3));

    printf("Test 2: calculate_something(10, 20, 3)\n");
    printf("Result: %d\n\n", calculate_something(10, 20, 3));

    printf("Test 3: factorial(5)\n");
    printf("Result: %d\n\n", factorial(5));

    printf("Test 4: safe_divide(10, 0) (error path)\n");
    printf("Result: %d\n\n", safe_divide(10, 0));

    /* Dump the raw events JSON FIRST. Both get_events and get_statistics DRAIN
     * the event store, so we fetch the JSON (which contains every traced
     * function name and the recorded exception) before anything consumes it. */
    char *events_json = xplainit_instrument_get_events();
    if (events_json != NULL) {
        printf("EVENTS_JSON_BEGIN\n%s\nEVENTS_JSON_END\n", events_json);
        xplainit_free_string(events_json);
    }

    /* Re-run a representative subset of the traced calls so that the store is
     * repopulated and get_statistics can report a non-zero, verifiable count. */
    (void)add(1, 2);
    (void)calculate_something(2, 3, 4);
    (void)factorial(4);

    unsigned long total = 0, functions = 0, errors = 0;
    xplainit_instrument_get_statistics(&total, &functions, &errors);

    printf("=================================================================\n");
    printf("Captured events summary (second batch):\n");
    printf("  total_events   = %lu\n", total);
    printf("  function_calls = %lu\n", functions);
    printf("  errors         = %lu\n", errors);
    printf("=================================================================\n\n");

    xplainit_instrument_shutdown();
    return 0;
}
