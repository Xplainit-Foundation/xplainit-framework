# Quickstart

This walks the paths that genuinely work today: producing a trace, then
rendering and analyzing it.

## 1. Produce a trace (Python)

The most complete live tracer is the Python module. After building it (see
[installation](installation.md)):

```python
import xplainit

xplainit.enable()          # install the sys.settrace hook
# ... run the code you want to explain ...
xplainit.disable()

events = xplainit.get_events()      # captured ExecutionEvent data
stats = xplainit.get_statistics()   # counts
```

You can persist `events` as a JSON array of `ExecutionEvent` values to feed the
CLI and dashboard. See the [Python guide](../guides/python.md) for the automatic
tracer, decorators, and frame filtering, and for the honest overhead note.

## 2. Render a trace with the CLI

Given a trace JSON file, the CLI renders it with the core formatters:

```sh
# text (default), json, html, or markdown
xplainit report trace.json --format text
xplainit report trace.json --format markdown
```

`xplainit run trace.json` also works: it detects a captured trace and renders
it. **`xplainit run some_program.py` does not live-trace the program** — it
prints exactly which binding to use to capture a trace first (see the
[introduction](../intro.md) for why).

## 3. Analyze errors

```sh
xplainit analyze trace.json
```

`analyze` runs the `ErrorExplainer` over the events and explains any error
events (root cause, suggestions). Secret-like values are redacted in all
rendered output by default.

## 4. Explain a specific function from a trace

```sh
xplainit explain app.py::main --trace trace.json
```

This explains the recorded events for `main`. Without `--trace`, live attach is
not wired.

## 5. Visualize in the dashboard

```sh
cargo run --release -p xplainit-dashboard -- trace.json
```

The dashboard loads the trace and serves a timeline, call graph, and summary on
fixed routes (`/`, `/summary`, `/callgraph`, `/timeline`, `/tasks`, `/events`).
It serves only these embedded routes (no dynamic file serving), enforces a
request-head size cap and a read timeout, and rejects non-GET requests.

## Using the Rust core directly

You can embed the engine in a Rust program:

```rust
use xplainit_core::{Config, Language, Explainer};

let config = Config::new(Language::Python);
let explainer = Explainer::new(config);
assert!(explainer.is_enabled());
```

From there, build events, push them through an `EventPipeline`
(filter → processor → sink), and render with a formatter. See
[concepts](concepts.md) and the [custom processors guide](../advanced/custom-processors.md).
