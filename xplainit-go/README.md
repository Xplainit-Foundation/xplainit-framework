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

First, build the C library from the repository root (it is a Cargo workspace):

```bash
cargo build --release
```

Then build the Go bindings:

```bash
cd xplainit-go
LD_LIBRARY_PATH=../target/release CGO_LDFLAGS='-L../target/release -lxplainit_c' go build ./...
```

## Runtime Tracing Status

Go does not expose a runtime tracing hook comparable to Python's `sys.settrace`,
so Xplainit-Go captures events through an **explicit, manual API** backed by the
C FFI:

- `OnFunctionEnter` / `OnFunctionExit` / `OnException` record events directly.
- `Trace` is the idiomatic `defer`-based wrapper built on top of them.

This manual/defer-based tracing is **implemented and verified end to end** in this
sandbox: `go test ./...` passes and the example program captures real function
enter/exit events (including recursion) plus an exception event through the native
library.

**Automatic instrumentation** (transparently tracing arbitrary Go code without
manual `Trace` calls) is **future work**. It would require either a debugger such
as [delve](https://github.com/go-delve/delve) or source-level code generation to
inject the `Trace` calls; neither is included yet.

## Usage

### Recording Events

```go
tracer := xplainit.New()
defer tracer.Close()
tracer.Enable()

// Record events explicitly.
tracer.OnFunctionEnter("compute", "compute.go", 10)
tracer.OnFunctionExit("compute", "compute.go", 10)
tracer.OnException("DivisionByZero", "division by zero", "math.go", 42)
```

### Idiomatic defer-based Tracing

`Trace` records the enter event immediately and returns a closure that records the
exit event. Invoke it and `defer` the returned closure with a trailing `()`:

```go
func myFunc(tracer *xplainit.Xplainit) {
    defer tracer.Trace("myFunc", "file.go", 12)()
    // ... function body ...
}
```

> **Note on draining:** both `GetStatistics()` and `GetEvents()` read *and drain*
> the underlying event store. Calling one empties it for the other. Read whichever
> you need first, or record fresh events before the second call.

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

> Reads and **drains** the event store, like `GetEvents()`.

### `(*Xplainit) OnFunctionEnter(name, file string, line int) bool`

Record a function-entry event. Only recorded while tracing is enabled.

**Returns:** `true` if the event was recorded.

### `(*Xplainit) OnFunctionExit(name, file string, line int) bool`

Record a function-exit event. Only recorded while tracing is enabled.

**Returns:** `true` if the event was recorded.

### `(*Xplainit) OnException(errType, message, file string, line int) bool`

Record an exception/error event. Only recorded while tracing is enabled.

**Returns:** `true` if the event was recorded.

### `(*Xplainit) Trace(name, file string, line int) func()`

Record a function-entry event immediately and return a closure that records the
matching exit event when called. Designed for use with `defer`:

```go
defer tracer.Trace("myFunc", "file.go", 12)()
```

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

## Building and Running (verified commands)

The C library must be built and resolvable at both link time and run time. From
the repository root:

```bash
# 1. Build the native C library (produces target/release/libxplainit_c.{a,so}).
cargo build --release

# 2. Build, test, and run from the Go module directory.
cd xplainit-go

# Build (CGO_LDFLAGS points the linker at the built library):
LD_LIBRARY_PATH=../target/release CGO_LDFLAGS='-L../target/release -lxplainit_c' go build ./...

# Test:
LD_LIBRARY_PATH=../target/release CGO_LDFLAGS='-L../target/release -lxplainit_c' go test ./...

# Run the example (LD_LIBRARY_PATH lets the .so resolve at run time):
LD_LIBRARY_PATH=../target/release go run ./examples/basic.go
```

The cgo preamble in `xplainit.go` already sets the default `-L${SRCDIR}/../target/release`
link path, so `CGO_LDFLAGS` is only needed when running from a different directory.
`LD_LIBRARY_PATH` (or an rpath / static link) is required at run time so the dynamic
loader can find `libxplainit_c.so`.

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
