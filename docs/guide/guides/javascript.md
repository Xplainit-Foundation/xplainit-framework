# JavaScript / Node

The Node binding (`xplainit-node`) is a native addon built with Neon (N-API).
It exposes an enable/disable/statistics-style API to JavaScript.

> **Offline note:** the development sandbox has **no `npm`** (only `bun` is
> present), so the Node addon **cannot be rebuilt or its JS test run** in that
> environment. The description here reflects the crate's API surface, not a
> live-verified build in this sandbox. See [`../../STATUS.md`](../../STATUS.md).

## Build (where npm is available)

The crate is a Neon addon with a `package.json`. Building follows the standard
Neon flow (a `cargo build`-backed `npm install`/build script produces the
`.node` addon), after which it can be `require`d from Node.

## API surface

The binding provides functions mirroring the core control surface, including
`enable()`, `disable()`, `isEnabled()`, `getEvents()`, `clearEvents()`, and
`getStatistics()`, plus TypeScript type definitions (`index.d.ts`).

```js
const xplainit = require('xplainit');

xplainit.enable();
// ... run code ...
xplainit.disable();

const events = xplainit.getEvents();
const stats = xplainit.getStatistics();
```

## Working with traces via the CLI

Regardless of how events are captured, once you have a JSON array of
`ExecutionEvent` values you can render and analyze it with the CLI:

```sh
xplainit report trace.json --format json
xplainit analyze trace.json
```

The CLI `run` command does not live-trace a `.js` file; it recognizes JavaScript
source and directs you to the Node binding to capture a trace first (see the
[introduction](../intro.md)).
