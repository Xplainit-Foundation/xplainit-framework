package main

import (
	"fmt"
	"github.com/xplainit/xplainit-go"
)

func fibonacci(n int) int {
	if n <= 1 {
		return n
	}
	return fibonacci(n-1) + fibonacci(n-2)
}

func main() {
	fmt.Println("Xplainit Go Example")
	fmt.Println("====================\n")

	// Create tracer
	fmt.Println("Creating Xplainit tracer...")
	tracer := xplainit.New()
	if tracer == nil {
		fmt.Println("Failed to create tracer")
		return
	}
	defer tracer.Close()

	// Get version
	fmt.Printf("Xplainit version: %s\n\n", xplainit.Version())

	// Enable tracing
	fmt.Println("Enabling tracing...")
	if tracer.Enable() {
		fmt.Println("Tracing enabled\n")
	}

	// Check if enabled
	if tracer.IsEnabled() {
		fmt.Println("Tracing is active\n")
	}

	// Your Go code would execute here
	result := fibonacci(5)
	fmt.Printf("fibonacci(5) = %d\n\n", result)

	// Get statistics
	fmt.Println("Getting statistics...")
	stats := tracer.GetStatistics()
	fmt.Printf("  Total events: %d\n", stats.TotalEvents)
	fmt.Printf("  Function calls: %d\n", stats.FunctionCalls)
	fmt.Printf("  Errors: %d\n\n", stats.Errors)

	// Get events
	fmt.Println("Getting events...")
	events := tracer.GetEvents()
	fmt.Printf("Events JSON: %s\n\n", events)

	// Clear events
	fmt.Println("Clearing events...")
	tracer.ClearEvents()

	// Disable tracing
	fmt.Println("Disabling tracing...")
	tracer.Disable()

	fmt.Println("\nExample completed successfully!")
}
