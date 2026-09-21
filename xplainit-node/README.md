# Xplainit Node.js

Natural language explanations for JavaScript/Node.js code execution.

## Status (Phase 2.2 - honest)

- **Rust/Neon addon builds:** `cargo build --release` produces
  `target/release/libxplainit_node.so`, and `cargo test -p xplainit-node`
  passes. The addon exports `enable`, `disable`, `isEnabled`, `getEvents`,
  `clearEvents`, `getStatistics`, and the tracer callbacks `onFunctionEnter`,
  `onFunctionExit`, `onException`.
- **Automatic JS tracer implemented and unit-verified:**
  `javascript/tracer.js` (`XplainitNodeTracer`) wraps functions, class methods,
  and objects to capture real argument values, real return values, and real
  exceptions (including `async`/`await` settlement), forwarding each to a
  backend implementing the callback contract above. It accepts an **injectable
  backend**, so its capture logic is exercised with a pure-JS mock in
  `test_automatic_tracing.js` (44 assertions across simple / recursion /
  exception / class-method / async cases). Run it with:
  `bun xplainit-node/test_automatic_tracing.js` (PASS/FAIL summary, non-zero
  exit on failure).
- **End-to-end native round-trip NOT verified in this sandbox.** Node.js and
  npm are not installed here (only `bun` 1.2.14). The Neon npm build pipeline
  (`cargo-cp-artifact` / `npm run build`) cannot run, and the N-API `.node`
  addon cannot be loaded under `bun` (`invalid ELF header` for the prebuilt
  `index.node`; the raw `.so` also fails to load). `node:inspector`'s `Session`
  *is* available under bun, but the wrapper tracer does not require it. To
  verify the JS -> Rust round-trip you need a real **Node.js + `npm run build`**
  install, then run `node test_automatic_tracing.js --native`.

## Installation

```bash
npm install xplainit
```

## Usage

### Module-level Functions

```javascript
const xplainit = require('xplainit');

// Enable tracing
xplainit.enable();

// Your JavaScript code here
function fibonacci(n) {
  if (n <= 1) return n;
  return fibonacci(n - 1) + fibonacci(n - 2);
}

fibonacci(5);

// Get captured events
const events = xplainit.getEvents();
console.log('Events:', JSON.parse(events));

// Get statistics
const stats = xplainit.getStatistics();
console.log('Statistics:', stats);
// Output: { total_events: 15, function_calls: 15, variable_operations: 0, errors: 0 }

// Clear events
xplainit.clearEvents();

// Disable tracing
xplainit.disable();
```

### Class-based API

```javascript
const { Xplainit } = require('xplainit');

const tracer = new Xplainit();

tracer.enable();

// Your code...
function add(a, b) {
  return a + b;
}

add(2, 3);

const events = tracer.getEvents();
console.log('Captured events:', JSON.parse(events));

tracer.disable();
```

### Automatic Tracer (wrapper-based)

`XplainitNodeTracer` instruments the functions/classes you hand it and forwards
live values to a backend. The backend is injectable: use the native addon in
production, or a mock in tests.

```javascript
const { XplainitNodeTracer, loadNativeBackend } = require('xplainit/javascript');

// Native addon in production (falls back to null if it cannot load):
const backend = loadNativeBackend();
const tracer = new XplainitNodeTracer(backend).enable();

// Wrap a plain function - captures args, return value, and exceptions:
const add = tracer.wrap((a, b) => a + b, { name: 'add', file: __filename });
add(2, 3); // -> onFunctionEnter('add', {arg0:'2', arg1:'3'}, ...), onFunctionExit('add', '5')

// Wrap all methods of a class (instance + static):
class Calculator {
  multiply(a, b) { return a * b; }
  static square(x) { return x * x; }
}
tracer.wrapClass(Calculator, { file: __filename });

// async / await: exit is recorded when the returned promise settles;
// a rejection is recorded via onException instead of onFunctionExit.
const fetchValue = tracer.wrap(async (x) => x * 2, { name: 'fetchValue' });
await fetchValue(21);
```

For testing without the native addon, inject any object implementing
`onFunctionEnter(name, args, file, line)`, `onFunctionExit(name, returnValue)`,
and `onException(errorType, message, file, line)`.

### TypeScript Support

```typescript
import * as xplainit from 'xplainit';
import type { Statistics } from 'xplainit';

xplainit.enable();

// Your TypeScript code
const result = someFunction();

const stats: Statistics = xplainit.getStatistics();
console.log(`Captured ${stats.total_events} events`);

xplainit.disable();
```

## API Reference

### `enable(): boolean`

Enable runtime tracing with default configuration.

**Returns:** `true` if successfully enabled

### `disable(): boolean`

Disable runtime tracing.

**Returns:** `true` if successfully disabled

### `isEnabled(): boolean`

Check if tracing is currently active.

**Returns:** `true` if enabled, `false` otherwise

### `getEvents(): string`

Get all captured execution events as a JSON string.

**Returns:** JSON string containing array of execution events

### `clearEvents(): boolean`

Clear all captured events from memory.

**Returns:** `true` if successfully cleared

### `getStatistics(): Statistics`

Get statistics about captured events.

**Returns:** Object with the following properties:
- `total_events`: Total number of captured events
- `function_calls`: Number of function call events
- `variable_operations`: Number of variable operations
- `errors`: Number of error events

### `class Xplainit`

Object-oriented interface for managing tracing.

#### `constructor()`

Create a new Xplainit tracer instance.

#### `enable(): void`

Enable tracing for this instance.

#### `disable(): void`

Disable tracing for this instance.

#### `getEvents(): string`

Get captured events as JSON string.

## Performance

The automatic tracer uses per-call function wrappers, so overhead scales with
the number of wrapped calls. When the tracer is disabled, wrapped functions are
a thin pass-through. Formal per-event benchmarks under a real Node.js runtime
have not yet been captured for this binding.

## License

MIT OR Apache-2.0
