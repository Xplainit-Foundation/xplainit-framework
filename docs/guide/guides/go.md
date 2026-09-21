# Go

The Go binding is a CGO wrapper around the C FFI library (`xplainit-c`). It
provides a Go-friendly `Xplainit` type over the same core control surface.

## Prerequisites

- Go (1.21+).
- The C FFI library built first, since the Go wrapper links against it:

```sh
cargo build --release -p xplainit-c
```

## API surface

The Go wrapper exposes an `Xplainit` struct and a `Statistics` struct, with
methods mirroring the C API: `New()`, `Close()`, `Enable()`, `Disable()`,
`IsEnabled()`, `GetEvents()`, `ClearEvents()`, `GetStatistics()`, and
`Version()`. CGO directives link the C library across platforms.

```go
x := xplainit.New()
defer x.Close()

x.Enable()
// ... run instrumented sections ...
x.Disable()

events := x.GetEvents()
stats := x.GetStatistics()
```

## Scope

This binding is a Go-callable control/recording surface over the C FFI, not a
full automatic source tracer for arbitrary Go programs. Capture events through
the API, export them as an `ExecutionEvent` JSON array, then use the CLI
(`report`, `analyze`) or the dashboard to render and analyze them.
