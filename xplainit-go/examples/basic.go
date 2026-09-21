package main

import (
	"errors"
	"fmt"

	xplainit "github.com/xplainit/xplainit-go"
)

// simpleGreeting is a plain function traced with the defer-Trace helper.
func simpleGreeting(tracer *xplainit.Xplainit, name string) string {
	defer tracer.Trace("simpleGreeting", "examples/basic.go", 12)()
	return "Hello, " + name
}

// factorial is a recursive function; each call records its own enter/exit
// pair, so recursion depth is visible in the captured events.
func factorial(tracer *xplainit.Xplainit, n int) int {
	defer tracer.Trace("factorial", "examples/basic.go", 19)()
	if n <= 1 {
		return 1
	}
	return n * factorial(tracer, n-1)
}

// riskyDivide traces itself and records an exception on the error path.
func riskyDivide(tracer *xplainit.Xplainit, a, b int) (int, error) {
	defer tracer.Trace("riskyDivide", "examples/basic.go", 28)()
	if b == 0 {
		err := errors.New("division by zero")
		// Go has no automatic exception hook, so we record the error
		// explicitly at the point it is detected.
		tracer.OnException("DivisionByZero", err.Error(), "examples/basic.go", 31)
		return 0, err
	}
	return a / b, nil
}

func main() {
	fmt.Println("Xplainit Go Example")
	fmt.Println("===================")
	fmt.Println()

	// Create tracer
	fmt.Println("Creating Xplainit tracer...")
	tracer := xplainit.New()
	if tracer == nil {
		fmt.Println("Failed to create tracer")
		return
	}
	defer tracer.Close()

	fmt.Printf("Xplainit version: %s\n\n", xplainit.Version())

	// Enable tracing (events are only recorded while enabled)
	fmt.Println("Enabling tracing...")
	if tracer.Enable() {
		fmt.Println("Tracing enabled")
	}
	fmt.Println()

	// 1. Simple function
	fmt.Println(simpleGreeting(tracer, "Xplainit"))

	// 2. Recursive function
	fmt.Printf("factorial(5) = %d\n", factorial(tracer, 5))

	// 3. Error path
	if _, err := riskyDivide(tracer, 10, 0); err != nil {
		fmt.Printf("riskyDivide(10, 0) failed: %v\n", err)
	}
	fmt.Println()

	// IMPORTANT: both GetStatistics and GetEvents DRAIN the event store in the
	// underlying engine, so calling one empties it for the other. Read
	// statistics first, then re-run the traced work to repopulate the store
	// before printing the raw events.
	fmt.Println("Statistics (reads and drains the event store):")
	stats := tracer.GetStatistics()
	fmt.Printf("  Total events:   %d\n", stats.TotalEvents)
	fmt.Printf("  Function calls: %d\n", stats.FunctionCalls)
	fmt.Printf("  Errors:         %d\n", stats.Errors)
	fmt.Println()

	// Re-run the traced work so there are fresh events to display.
	simpleGreeting(tracer, "again")
	factorial(tracer, 3)
	_, _ = riskyDivide(tracer, 1, 0)

	fmt.Println("Events JSON (reads and drains the event store):")
	events := tracer.GetEvents()
	fmt.Println(events)
	fmt.Println()

	fmt.Println("Disabling tracing...")
	tracer.Disable()

	fmt.Println("Example completed successfully!")
}
