/*
 * Xplainit C Example
 * 
 * Demonstrates using Xplainit from C code.
 */

#include <stdio.h>
#include <stdlib.h>
#include "../include/xplainit-c.h"

int main(void) {
    printf("Xplainit C Example\n");
    printf("==================\n\n");

    // Create a new Xplainit runtime
    printf("Creating Xplainit runtime...\n");
    XplainitHandle* handle = xplainit_create();
    if (handle == NULL) {
        fprintf(stderr, "Failed to create Xplainit handle\n");
        return 1;
    }

    // Get version
    const char* version = xplainit_version();
    printf("Xplainit version: %s\n\n", version);

    // Enable tracing
    printf("Enabling tracing...\n");
    if (!xplainit_enable(handle)) {
        fprintf(stderr, "Failed to enable tracing\n");
        xplainit_free(handle);
        return 1;
    }

    // Check if enabled
    if (xplainit_is_enabled(handle)) {
        printf("Tracing is enabled\n\n");
    }

    // Your C code would execute here and generate events
    // For demonstration, we'll just work with the empty event list

    // Get statistics
    printf("Getting statistics...\n");
    size_t total_events = 0;
    size_t function_calls = 0;
    size_t errors = 0;
    
    if (xplainit_get_statistics(handle, &total_events, &function_calls, &errors)) {
        printf("  Total events: %zu\n", total_events);
        printf("  Function calls: %zu\n", function_calls);
        printf("  Errors: %zu\n\n", errors);
    }

    // Get events as JSON
    printf("Getting events...\n");
    char* events_json = xplainit_get_events(handle);
    if (events_json != NULL) {
        printf("Events JSON: %s\n\n", events_json);
        xplainit_free_string(events_json);
    }

    // Clear events
    printf("Clearing events...\n");
    xplainit_clear_events(handle);

    // Disable tracing
    printf("Disabling tracing...\n");
    xplainit_disable(handle);

    // Clean up
    printf("Cleaning up...\n");
    xplainit_free(handle);

    printf("\nExample completed successfully!\n");
    return 0;
}
