# C / C++

The C binding (`xplainit-c`) exposes a C-compatible FFI over the core engine.
It builds as both a dynamic (`cdylib`) and static (`staticlib`) library, and a C
header is generated with `cbindgen`. C++ can use the same C API (an RAII wrapper
example is provided).

## Build

```sh
cargo build --release -p xplainit-c
```

This produces the shared/static libraries; the generated header (`xplainit-c.h`)
declares the exported functions.

## API surface

The FFI exposes lifecycle and control entry points such as `xplainit_create()`,
`xplainit_free()`, `xplainit_enable()`, `xplainit_disable()`, and related
event/statistics accessors. Link against the built library and include the
generated header.

```c
#include "xplainit-c.h"

int main(void) {
    void *x = xplainit_create();
    xplainit_enable(x);
    /* ... run instrumented sections ... */
    xplainit_disable(x);
    xplainit_free(x);
    return 0;
}
```

(Consult the generated `xplainit-c.h` for the exact, current signatures.)

## Safety

All `unsafe` FFI blocks in this crate carry `// SAFETY:` comments documenting
their invariants, and raw-pointer entry points guard against null pointers
before dereferencing (Phase 4, Task 4.1). See
[`../../SECURITY_AUDIT.md`](../../SECURITY_AUDIT.md).

## Scope

This binding is a C-callable control/recording surface over the core, not a
full automatic source tracer for arbitrary C/C++ programs. Capture events
through the API, export them as an `ExecutionEvent` JSON array, then use the CLI
(`report`, `analyze`) or dashboard to render and analyze them.
