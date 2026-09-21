# xplainit-dashboard

A single-binary web server that loads an Xplainit trace and serves an
interactive dashboard for real-time trace visualization: a timeline, a call
graph, and an aggregated error summary.

The server is built directly on tokio's TCP primitives with a small
hand-written HTTP layer. No external web framework is used, and the event
stream is delivered over Server-Sent Events (SSE), which needs no handshake and
no extra dependency. Error analysis is delegated to
`xplainit_core::ErrorExplainer`, so the dashboard does not reimplement any error
reasoning.

## Running the server

The server takes a trace JSON file: a JSON array of
`xplainit_core::ExecutionEvent` values using the same externally tagged serde
shape the core engine produces and the CLI reads (one single-key object per
event keyed by the variant name, for example `{"FunctionEnter": {...}}`).

```sh
cargo run -p xplainit-dashboard -- <trace.json>
```

Optional flags:

| Flag                  | Default          | Description                                            |
| --------------------- | ---------------- | ------------------------------------------------------ |
| `--bind <ADDR>`       | `127.0.0.1:8080` | Address to bind the HTTP server to.                    |
| `--stream-delay-ms N` | `0`              | Delay in milliseconds between streamed events (replay pacing). |

Example:

```sh
cargo run -p xplainit-dashboard -- trace.json --bind 127.0.0.1:8091 --stream-delay-ms 200
```

Then open http://127.0.0.1:8091/ in a browser.

## Endpoints

| Method / path      | Response                                                                 |
| ------------------ | ------------------------------------------------------------------------ |
| `GET /`, `/index.html` | Dashboard page (HTML).                                               |
| `GET /style.css`   | Stylesheet (CSS).                                                        |
| `GET /app.js`      | Frontend script (vanilla JS).                                            |
| `GET /summary`     | Aggregated error summary (JSON): `total_events`, `error_count`, `counts_by_type`, and per-error `analyses` from `ErrorExplainer`. |
| `GET /callgraph`   | Derived call graph (JSON): `nodes` with `name`, `depth`, `parent`, `children`, built from `FunctionEnter`/`FunctionExit` nesting. |
| `GET /timeline`    | Derived timeline (JSON): ordered `events` with `event_type`, `is_error`, `location`, `timestamp`. |
| `GET /tasks`       | Per async-task view (JSON): `task_count` and `tasks`, each with `task_id`, `task_name`, `state`, `event_count`, and its ordered lifecycle `events`, grouped by `task_id` via `xplainit_core::AsyncTaskTracker`. |
| `GET /events`      | SSE stream of the trace events (one `data:` message per event), terminated by `{"done":true}`. |

Any other path returns `404 Not Found`. Non-`GET` methods return
`405 Method Not Allowed`. Request heads larger than 64 KiB return
`431 Request Header Fields Too Large`.

The connection reader accumulates the request head across multiple TCP reads
(so requests split across segments or larger than one read buffer parse
correctly) and applies a 15-second per-connection read timeout, so an idle or
slow client cannot pin a spawned task indefinitely.

## Frontend

The shipped, working default frontend is hand-written static assets in
`static/` (`index.html`, `style.css`, `app.js`, vanilla JavaScript). It is
compiled into the binary with `include_str!`, so there is **no build step**: it
works as soon as the Rust binary is built. The page connects to `GET /events`
via `EventSource` and builds the live timeline from the streamed events, and
fetches `GET /summary` (error panel) and `GET /callgraph` (call-graph tree) with
`fetch`. The standalone `GET /timeline` endpoint exposes the same timeline data
as JSON for other consumers.

## Sandbox status

- **Rust server + default static frontend: fully buildable and runnable here.**
  `cargo build --release -p xplainit-dashboard`, `cargo test`, `cargo clippy`,
  and `cargo fmt` all pass, and the server has been smoke-tested end to end
  (static page, `/summary`, `/callgraph`, `/timeline`, and the `/events` SSE
  stream all return real output). It uses only dependencies already present in
  `Cargo.lock` (tokio, serde_json, anyhow, clap, xplainit-core); no new crates
  were added.
- **Any npm/React/D3 frontend variant is NOT buildable in this sandbox.**
  `node`/`npm` are absent, so a JS bundler toolchain cannot run here. No such
  variant is shipped or claimed to work. The vanilla-JS static frontend is the
  supported default and needs no toolchain beyond the Rust compiler.
