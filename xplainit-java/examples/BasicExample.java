package io.xplainit.examples;

import io.xplainit.Xplainit;
import io.xplainit.Xplainit.Statistics;

/**
 * Basic Xplainit usage example in Java
 */
public class BasicExample {
    
    public static void main(String[] args) {
        System.out.println("Xplainit Java Example");
        System.out.println("=====================\n");
        
        // Create tracer with try-with-resources (auto-close)
        try (Xplainit tracer = new Xplainit()) {
            
            System.out.println("Enabling tracing...");
            tracer.enable();
            
            if (tracer.isEnabled()) {
                System.out.println("Tracing is enabled\n");
            }
            
            // Your Java code would execute here and generate events
            // For demonstration, we'll just work with the empty event list
            fibonacci(5);
            
            // Get statistics
            System.out.println("Getting statistics...");
            Statistics stats = tracer.getStatistics();
            System.out.println("  " + stats + "\n");
            
            // Get events as JSON
            System.out.println("Getting events...");
            String events = tracer.getEvents();
            System.out.println("Events JSON: " + events + "\n");
            
            // Clear events
            System.out.println("Clearing events...");
            tracer.clearEvents();
            
            // Disable tracing
            System.out.println("Disabling tracing...");
            tracer.disable();
            
            System.out.println("\nExample completed successfully!");
            
        } catch (Exception e) {
            System.err.println("Error: " + e.getMessage());
            e.printStackTrace();
        }
    }
    
    /**
     * Simple fibonacci function for demonstration
     */
    private static int fibonacci(int n) {
        if (n <= 1) {
            return n;
        }
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}
