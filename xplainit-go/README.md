# Xplainit Go

Natural language explanations for Go code execution.

## Requirements

- Go 1.21 or higher
- CGO enabled
- Xplainit C library built (`xplainit-c`)

## Installation

```bash
go get github.com/xplainit/xplainit-go
```

## Building

First, build the C library:

```bash
cd ../xplainit-c
cargo build --release
```

Then you can use the Go bindings:

```bash
cd ../xplainit-go
go build
```

## Usage

### Basic Usage

```go
package main

import (
    "fmt"
    "github.com/xplainit/xplainit-go"
)

func main() {
    // Create tracer
    tracer := xplainit.New()
    if tracer == nil {
        panic("Failed to create tracer")
    }
    defer tracer.Close()
    
    // Enable tracing
    tracer.Enable()
    
    // Your code here
    result := fibonacci(10)
    fmt.Println("Result:", result)
    
    // Get statistics
    stats := tracer.GetStatistics()
    fmt.Printf("Captured %d events\n", stats.TotalEvents)
    fmt.Printf("Function calls: %d\n", stats.FunctionCalls)
    fmt.Printf("Errors: %d\n", stats.Errors)
    
    // Get events as JSON
    events := tracer.GetEvents()
    fmt.Println("Events:", events)
    
    // Disable tracing
    tracer.Disable()
}

func fibonacci(n int) int {
    if n <= 1 {
        return n
    }
    return fibonacci(n-1) + fibonacci(n-2)
}
```

### Using defer for Cleanup

```go
func processData() error {
    tracer := xplainit.New()
    if tracer == nil {
        return errors.New("failed to create tracer")
    }
    defer tracer.Close() // Automatically cleanup
    
    tracer.Enable()
    defer tracer.Disable() // Automatically disable
    
    // Your code here...
    
    return nil
}
```

## API Reference

### `New() *Xplainit`

Creates a new Xplainit tracer instance.

**Returns:** Tracer instance or `nil` on failure

**Example:**
```go
tracer := xplainit.New()
defer tracer.Close()
```

### `(*Xplainit) Enable() bool`

Enable runtime tracing.

**Returns:** `true` if successfully enabled

### `(*Xplainit) Disable() bool`

Disable runtime tracing.

**Returns:** `true` if successfully disabled

### `(*Xplainit) IsEnabled() bool`

Check if tracing is currently active.

**Returns:** `true` if enabled, `false` otherwise

### `(*Xplainit) GetEvents() string`

Get all captured events as a JSON string.

**Returns:** JSON array of events

### `(*Xplainit) ClearEvents() bool`

Clear all captured events from memory.

**Returns:** `true` if successfully cleared

### `(*Xplainit) GetStatistics() *Statistics`

Get statistics about captured events.

**Returns:** Pointer to Statistics struct

### `(*Xplainit) Close() error`

Free native resources. Should be called with `defer`.

**Returns:** Always returns `nil`

### `Version() string`

Get the Xplainit version string.

**Returns:** Version string (e.g., "0.1.0")

## Statistics Struct

```go
type Statistics struct {
    TotalEvents   int // Total number of captured events
    FunctionCalls int // Number of function call events
    Errors        int // Number of error events
}
```

## Examples

See the `examples/` directory:

- `basic.go` - Simple usage demonstration

## Building Examples

```bash
# Build the C library first
cd ../xplainit-c
cargo build --release

# Build and run the Go example
cd ../xplainit-go
go run examples/basic.go
```

## Environment Variables

### Linux/macOS

```bash
export LD_LIBRARY_PATH=$PWD/../target/release:$LD_LIBRARY_PATH
export DYLD_LIBRARY_PATH=$PWD/../target/release:$DYLD_LIBRARY_PATH
go run examples/basic.go
```

### Windows

```powershell
$env:PATH = "$PWD\..\target\release;$env:PATH"
go run examples/basic.go
```

## Performance

Xplainit is designed for minimal overhead:
- **<2μs per event** on modern hardware
- **1-2% overhead** for typical applications
- **Zero-cost** when disabled

## Thread Safety

All methods are thread-safe. The same tracer instance can be used across multiple goroutines.

## CGO Requirements

This package uses CGO to interface with the native Xplainit C library. Ensure:

1. CGO is enabled: `export CGO_ENABLED=1`
2. C compiler is available (gcc, clang, or MSVC)
3. Xplainit C library is built

## Troubleshooting

### "undefined reference" errors

Ensure the C library is built:
```bash
cd ../xplainit-c
cargo build --release
```

### "cannot find -lxplainit_c"

Set the library path:
```bash
export LD_LIBRARY_PATH=$PWD/../target/release:$LD_LIBRARY_PATH
```

### Cross-compilation

When cross-compiling, build the C library for the target platform first, then build the Go bindings with the appropriate `GOOS` and `GOARCH`.

## License

MIT OR Apache-2.0
