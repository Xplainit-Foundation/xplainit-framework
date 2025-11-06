# Xplainit Node.js

Natural language explanations for JavaScript/Node.js code execution.

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

Xplainit is designed for minimal overhead:
- **<2μs per event** on modern hardware
- **1-2% overhead** for typical applications
- **Zero-cost** when disabled

## License

MIT OR Apache-2.0
