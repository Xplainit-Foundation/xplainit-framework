# Xplainit VS Code Extension

A Visual Studio Code extension for the Xplainit execution-tracing framework. It
consumes the same JSON trace format that `xplainit-core` emits and the
`xplainit` CLI reads, and turns those traces into an interactive debugging
experience inside the editor.

## Features

- **Inline hover explanations.** Hover over any source line and see the
  Xplainit events that were recorded there (function entries, variable
  assignments, conditional evaluations, errors, and more), matched by
  `SourceLocation` file and line.
- **Step-through visualization.** A "Xplainit Timeline" tree view in the
  Explorer lists every event in trace order. Playback highlights the
  corresponding source line and lets you step forward and backward.
- **Error analysis panel.** A webview that lists every error event in the trace
  (exception, runtime error, type error, null-pointer access, index out of
  bounds, division by zero, stack overflow, panic, infinite loop, deadlock,
  memory leak) and renders the output of `xplainit analyze` for deeper,
  ErrorExplainer-backed analysis.
- **Trace record and playback.** "Record" shells out to the `xplainit` CLI to
  produce a trace JSON file, then loads it. "Playback" steps through the loaded
  events, highlighting the source line for each one.

## Commands

All commands are available from the Command Palette under the `Xplainit` category:

- `Xplainit: Load Trace File` (`xplainit.loadTrace`)
- `Xplainit: Record Trace` (`xplainit.recordTrace`)
- `Xplainit: Playback Trace` (`xplainit.playbackTrace`)
- `Xplainit: Step Forward` (`xplainit.stepForward`)
- `Xplainit: Step Backward` (`xplainit.stepBackward`)
- `Xplainit: Stop Playback` (`xplainit.stopPlayback`)
- `Xplainit: Explain Function at Cursor` (`xplainit.explainFunction`)
- `Xplainit: Show Error Analysis Panel` (`xplainit.showErrorPanel`)

## Settings

- `xplainit.cliPath` (default `xplainit`): path to the `xplainit` CLI binary
  used for `record`, `analyze`, and `report`.
- `xplainit.recordCommand` (default `run`): the CLI subcommand used to record a
  trace. This matches the FEAT-001 CLI, whose subcommands are `run`, `explain`,
  `analyze`, and `report`.

## Trace format

The extension reads a JSON array of `ExecutionEvent` values exactly as
`xplainit-core` serializes them. The Rust `ExecutionEvent` enum uses serde's
default externally-tagged representation, so each event is a single-key object
whose key is the variant name:

```json
[
  {
    "FunctionEnter": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "divide",
      "args": {},
      "location": { "file": "app.py", "line": 10, "column": 1, "offset": 0 },
      "timestamp": "2024-01-01T00:00:00Z"
    }
  },
  {
    "DivisionByZero": {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "numerator": { "Integer": 10 },
      "denominator_var": "x",
      "location": { "file": "app.py", "line": 12, "column": 5, "offset": 0 },
      "timestamp": "2024-01-01T00:00:01Z"
    }
  }
]
```

The runtime `Value` type is also externally tagged (for example
`{ "Integer": 10 }`, `{ "String": "hi" }`), and unit variants such as `Null`
serialize as the bare string `"Null"`. The TypeScript interfaces in
`src/trace.ts` mirror these shapes, and the error-variant set and snake_case
`event_type()` discriminators are kept in sync with
`xplainit-core/src/events.rs`.

Trace files may use the `.xplainit-trace` extension (a TextMate grammar in
`syntaxes/xplainit-trace.json` provides basic JSON-with-event-variant syntax
highlighting) or a plain `.json` extension.

## Build, run, and package

This is a standalone TypeScript project. It is intentionally **not** part of the
Rust Cargo workspace and does not affect `cargo build` or `cargo test`.

```sh
cd xplainit-vscode

# 1. Install dependencies (vscode API types, TypeScript compiler).
npm install

# 2. Compile TypeScript to ./out.
npm run compile        # or: npm run watch  for incremental builds

# 3. Launch the Extension Development Host.
#    Open this folder in VS Code and press F5 (uses .vscode/launch.json),
#    or run the "Run Extension" launch configuration.

# 4. Package a .vsix for distribution (requires the vsce tool).
npm install -g @vscode/vsce
vsce package
```

## Sandbox status

Honest note about this environment: the TypeScript source and manifest in this
directory were **authored but not compiled, packaged, run, or tested here.**

The build environment for this task runs under an INTEGRATIONS_ONLY network
policy with repository access only, and it has **no Node.js / npm toolchain**
(and no access to the npm registry or the VS Code marketplace / `vsce`).
Consequently the following steps could not be executed in this sandbox and are
therefore documented rather than claimed as passing:

- `npm install` (no npm and no registry access; `@types/vscode`, `@types/node`,
  and `typescript` could not be downloaded).
- `npm run compile` / `tsc` (no TypeScript compiler available to install).
- `F5` Extension Development Host launch (requires a full VS Code + Node setup).
- `vsce package` (the `@vscode/vsce` tool and marketplace are unreachable).

To verify and package the extension, run `npm install && npm run compile` in an
environment that has Node.js, npm, and network access to the npm registry, then
use `vsce package` to build a `.vsix`. The source has been hand-checked for
type and API consistency against the documented `vscode` extension API, but a
real `tsc` type-check should be run in that environment as the source of truth.
