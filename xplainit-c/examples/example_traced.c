/**
 * Example: C Program with Automatic Tracing
 * 
 * Compile with:
 *   gcc -finstrument-functions -rdynamic -DXPLAINIT_DEBUG=1 \
 *       example_traced.c ../lib/trace.c -lpthread -o example_traced
 * 
 * Run with:
 *   XPLAINIT_DEBUG=1 ./example_traced
 */

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

// Simple functions to trace
int add(int a, int b) {
    return a + b;
}

int multiply(int x, int y) {
    return x * y;
}

// Recursive function
int factorial(int n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

// Function with loops
void process_array(int* arr, int size) {
    for (int i = 0; i < size; i++) {
        arr[i] = arr[i] * 2;
    }
}

// Nested function calls
int calculate_something(int a, int b, int c) {
    int sum = add(a, b);
    int product = multiply(sum, c);
    return product;
}

int main() {
    printf("=================================================================\n");
    printf("Xplainit C/C++ Automatic Tracing Example\n");
    printf("=================================================================\n\n");
    
    printf("Running traced functions...\n\n");
    
    // Test 1: Simple function
    printf("Test 1: add(5, 3)\n");
    int result1 = add(5, 3);
    printf("Result: %d\n\n", result1);
    
    // Test 2: Nested calls
    printf("Test 2: calculate_something(10, 20, 3)\n");
    int result2 = calculate_something(10, 20, 3);
    printf("Result: %d\n\n", result2);
    
    // Test 3: Recursive function
    printf("Test 3: factorial(5)\n");
    int result3 = factorial(5);
    printf("Result: %d\n\n", result3);
    
    // Test 4: Array processing
    printf("Test 4: process_array\n");
    int arr[] = {1, 2, 3, 4, 5};
    process_array(arr, 5);
    printf("Result: [");
    for (int i = 0; i < 5; i++) {
        printf("%d%s", arr[i], i < 4 ? ", " : "");
    }
    printf("]\n\n");
    
    printf("=================================================================\n");
    printf("Tracing complete!\n");
    printf("=================================================================\n\n");
    
    printf("Check stderr for trace output (with XPLAINIT_DEBUG=1)\n");
    printf("\nAll function entries and exits were automatically traced!\n");
    printf("No manual instrumentation required!\n\n");
    
    return 0;
}
