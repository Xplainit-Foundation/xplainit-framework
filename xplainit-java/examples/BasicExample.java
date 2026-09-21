package io.xplainit.examples;

import io.xplainit.Xplainit;

/**
 * Basic Xplainit usage example in Java.
 *
 * <p>This example is designed to be run under the JVMTI agent:
 *
 * <pre>
 *   java -agentpath:$PWD/libxplainit_agent.so \
 *        -cp classes:examples io.xplainit.examples.BasicExample
 * </pre>
 *
 * The agent traces methods in the {@code io.xplainit.examples} package and
 * records MethodEntry / MethodExit / Exception events into the process-wide
 * Xplainit runtime. At the end we read the captured events back and print them.
 */
public class BasicExample {

    public static void main(String[] args) {
        System.out.println("Xplainit Java Example");
        System.out.println("=====================");

        // Simple method call.
        int doubled = doubleValue(21);
        System.out.println("doubleValue(21) = " + doubled);

        // Recursive method.
        int fib = fibonacci(6);
        System.out.println("fibonacci(6) = " + fib);

        // Exception-throwing method (caught here so the example still exits 0).
        try {
            divide(10, 0);
        } catch (ArithmeticException e) {
            System.out.println("Caught expected exception: " + e.getMessage());
        }

        // Read back what the agent captured. These statics live in io.xplainit
        // (NOT io.xplainit.examples), so they are not themselves traced.
        // IMPORTANT: getStatistics and getEvents both DRAIN the store, so grab
        // the events JSON first, then compute stats from that same snapshot.
        String events = Xplainit.agentGetEvents();

        System.out.println("\n--- Captured events (JSON) ---");
        System.out.println(events);

        System.out.println("\n--- Summary ---");
        System.out.println("captured_methods_present="
            + (events.contains("doubleValue")
               && events.contains("fibonacci")
               && events.contains("divide")));
        System.out.println("event_count_nonzero=" + (!events.equals("[]") && !events.isEmpty()));

        System.out.println("\nExample completed successfully!");
    }

    /** Simple method for demonstration. */
    private static int doubleValue(int n) {
        return n * 2;
    }

    /** Recursive fibonacci for demonstration. */
    private static int fibonacci(int n) {
        if (n <= 1) {
            return n;
        }
        return fibonacci(n - 1) + fibonacci(n - 2);
    }

    /** Exception-throwing method for demonstration. */
    private static int divide(int a, int b) {
        return a / b;
    }
}
