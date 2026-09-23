# Xplainit

Xplainit is an experimental, **alpha-stage** framework for explaining what a
program does at runtime, in plain language, without using any AI, ML, or network
calls. It hooks into a running program, records function calls, returns, and
exceptions, and turns that trace into human-readable descriptions and error
analysis.

It is a research/hobby project, not a finished product. The Rust core is solid
and well tested; the language bindings vary a lot in how complete and usable they
are. This README tries to be honest about exactly where each part stands so you
don't waste time expecting something that isn't there yet.

Nothing here is published to any package registry. You build it from source.

## What it actually is

- A Rust **core engine** (`xplainit-core`) that models execution events (function
  enter/exit, exceptions, etc.), runs them through a filter + processing
  pipeline, and generates explanations and error analysis. It also does source
  parsing with tree-sitter, secret redaction, and has a circuit breaker for
  runtime safety.
- A set of **language tracers** that feed real programs' execution into that
  core: Python, C, Java, and Go. Quality differs a lot between them (see the
  support table).
- A **CLI** (`xplainit`) that analyzes and reports on captured trace files.
- A **web dashboard** (`xplainit-dashboard`) that serves a trace over HTTP with a
  no-build static frontend and an SSE event stream.

## Honest status

Everything below was verified by building and running it on a normal Linux dev
machine. The workspace gates currently pass:

- `cargo build --release` — builds all 7 workspace crates.
- `cargo test` — **254 passed, 0 failed, 1 ignored**. The one ignored test is a
  1-million-event soak test; it passes when you run it explicitly with
  `-- --ignored`.
- `cargo clippy --all-targets --all-features -- -D warnings` — clean (exit 0).
- `cargo fmt --all -- --check` — clean (exit 0).
- `cargo doc --all --no-deps` — no warnings.

### The overhead reality

Full automatic Python tracing uses `sys.settrace`, and that is **slow**. On a
recursive `fib(25)` benchmark on this machine, tracing everything made the run
about **1100x slower** (measured: ~12 ms untraced vs ~13 s traced). That is not a
typo and it is not a bug you can optimize away: `sys.settrace` fires a Python
callback on every single frame event, and that call overhead dominates.

This is the same ballpark as other pure-`settrace` tools like PySnooper. If you
need low overhead, you do **not** trace everything: you use selective tracing
(decorate specific functions, or trace a narrow scope). Any claim of "1-2%
overhead" or "<10% overhead" for full automatic tracing is false. Don't believe
it, including from older versions of this very README.

## Language support, honestly

| Target | State | Notes |
|--------|-------|-------|
| Rust core (`xplainit-core`) | **Works** | Engine, filters, pipeline, explanations, error analysis, tree-sitter parsing, redaction, circuit breaker. Well tested. |
| Python (`xplainit-python`) | **Works** | Automatic tracing via `sys.settrace` (real recursion + exception capture) and manual/decorator tracing. Huge overhead when tracing everything (see above). |
| C (`xplainit-c`) | **Works, limited** | Uses `-finstrument-functions` + `dladdr`. Captures calls/returns/exceptions, but **no argument values** and **no per-call line numbers**. |
| Java (`xplainit-java`) | **Works, limited** | JVMTI agent via `-agentpath`. Same limits as C: no arg values, no line numbers. Reports **raw JVMTI type signatures** (e.g. `Ljava/lang/ArithmeticException;`) and the exception **message is empty**. |
| Go (`xplainit-go`) | **Works, but manual** | There is no automatic hook. You add explicit `defer`-based trace calls yourself. Requires `LD_LIBRARY_PATH` pointing at the built C shared library. Not a workspace member; built separately. |
| CLI (`xplainit-cli`) | **Works, partial** | `analyze` and `report` work on a trace JSON file. `run` and `explain` do **not** do live attach; they say so and point you to capture a trace via a binding first. |
| Dashboard (`xplainit-dashboard`) | **Works** | Tokio HTTP server, SSE event stream, static frontend, path-traversal-safe router. |
| Node (`xplainit-node`) | **Unverified / likely broken as shipped** | The pure-JS tracer logic passes its tests against a mock backend, but the native JS↔Rust bridge is **not verified end to end** here. The shipped `index.node` is a **Windows DLL**, and this environment has no node/npm to rebuild it. Treat it as not usable yet. |
| VS Code extension (`xplainit-vscode`) | **Source only, unbuilt** | A real TypeScript extension, but it needs `npm` to build/package, which isn't available here. Unverified. |
| Rust proc-macro tracing | **Does not exist** | Older docs advertised this. There is no such feature. |

## Building

You need a recent Rust toolchain (built and tested with rustc 1.92).

```bash
git clone <this-repo>
cd xplainit-framework
cargo build --release
cargo test          # 254 passed, 0 failed, 1 ignored
```

The CLI binary lands at `target/release/xplainit` and the dashboard at
`target/release/xplainit-dashboard`.

### Python

There is no `pip install xplainit`. Build the extension from source with
[maturin](https://www.maturin.rs/):

```bash
python -m venv .venv
source .venv/bin/activate
pip install maturin
cd xplainit-python
maturin develop --release
```

After that, `import xplainit` works in that virtualenv.

### C, Java, Go

These build on top of the C shared library. See each crate's own README and
build script:

- `xplainit-c/examples/build.sh`
- `xplainit-java/build_agent.sh`
- `xplainit-go/` (run its tests/examples with
  `LD_LIBRARY_PATH=../target/release`)

## Python quick start

These examples use the **real, current** API and have been run to confirm they
work.

### Automatic tracing (`sys.settrace`)

```python
import xplainit

def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

tracer = xplainit.AutoTracer()
tracer.start()          # installs sys.settrace under the hood
fib(5)
tracer.stop()

events = tracer.get_events()   # returns a Python list of event dicts
print(len(events), "events")
print(tracer.get_stats())      # e.g. "Events captured: 30, Enabled: false"
```

Remember: this traces *everything* while active, so it is very slow on hot code.
Keep the traced region small.

### Selective tracing with a decorator

```python
import json
import xplainit

@xplainit.explain_function
def add(a, b):
    return a + b

add(2, 3)   # -> 5, and recorded

# The decorator records into a shared backend you can inspect:
backend = xplainit.explain_backend()
events = json.loads(backend.get_events())   # get_events() returns a JSON string here
print(events)
```

### The core backend directly

```python
import xplainit

x = xplainit.Xplainit()          # backend object
print(x.get_stats())             # "Events captured: 0, Enabled: true"
print(x.get_events())            # JSON string: "[]" when empty
```

Note the small inconsistency worth knowing: `AutoTracer.get_events()` returns a
**list**, while `Xplainit.get_events()` (and `explain_backend().get_events()`)
returns a **JSON string**. The stats method is `get_stats()` and returns a
string, not a dict.

## CLI

The CLI works on a trace JSON file (capture one with a language binding first):

```bash
target/release/xplainit analyze path/to/trace.json   # root cause + fix suggestions
target/release/xplainit report  path/to/trace.json   # renders calls + exceptions
```

`xplainit run <file>` and `xplainit explain <path::fn>` intentionally do **not**
attach to a live process; they will tell you to capture a trace via a binding
instead.

## Dashboard

```bash
target/release/xplainit-dashboard path/to/trace.json --bind 127.0.0.1:8791
```

Then open the bound address. Endpoints include `/`, `/summary`, `/callgraph`,
`/timeline`, `/tasks`, and `/events` (a Server-Sent Events stream). The router is
a fixed allowlist, so path traversal is not possible.

## Repository layout

- `xplainit-core/` — the Rust engine: events, filters, pipeline, explanations,
  error analysis, tree-sitter parsing, redaction, circuit breaker. This is the
  most complete and best-tested part.
- `xplainit-python/` — PyO3 bindings; automatic (`sys.settrace`) and manual
  tracing. See its own README (`xplainit-python/README.md`).
- `xplainit-c/` — C FFI tracer using `-finstrument-functions`.
- `xplainit-java/` — JVMTI agent + JNI bridge.
- `xplainit-go/` — manual, `defer`-based Go tracer over the C library (not a
  Cargo workspace member).
- `xplainit-node/` — Neon-based Node binding. Unverified end to end; see the
  status table.
- `xplainit-cli/` — the `xplainit` command-line tool.
- `xplainit-dashboard/` — the HTTP/SSE trace dashboard.
- `xplainit-vscode/` — a TypeScript VS Code extension (source only here).
- `docs/` — additional documentation.

## Roadmap and contributing

- Roadmap and the plan for getting to a genuinely production-ready state:
  [PRODUCTION_READINESS_PLAN.md](PRODUCTION_READINESS_PLAN.md) and
  [FRAMEWORK_PLAN.md](FRAMEWORK_PLAN.md).
- Design notes: [DESIGN_PHILOSOPHY.md](DESIGN_PHILOSOPHY.md) and
  [ERROR_HANDLING_SYSTEM.md](ERROR_HANDLING_SYSTEM.md).
- How to contribute: [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under the MIT License. See [LICENSE](LICENSE) (and the identical
[LICENSE-MIT](LICENSE-MIT)). Some older docs mention a dual MIT/Apache-2.0
license, but there is no Apache license file in this repository, so treat it as
MIT only unless that changes.
