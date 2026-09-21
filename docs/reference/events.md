# Reference: Events and Values

This page documents the event model that flows through the framework. Every
statement here is grounded in `xplainit-core/src/events.rs`.

An execution trace is an ordered array of `ExecutionEvent` values. Each event is
`serde`-serializable (JSON is the canonical wire format used by the CLI and
dashboard), carries a unique `id` (`uuid::Uuid`) and a `timestamp`
(`chrono::DateTime<Utc>`), and most variants carry a `SourceLocation`.

## `SourceLocation`

```text
SourceLocation { file: String, line: usize, column: usize, offset: usize }
```

- `SourceLocation::new(file, line, column)` sets `offset` to 0.
- `SourceLocation::unknown()` yields `file = "<unknown>"`, all positions 0. It is
  the fallback returned by `ExecutionEvent::location()` for variants that carry
  no location of their own.

## `Value`

`Value` is the runtime value representation captured for arguments, variables,
return values, and error payloads:

| Variant           | Payload            | `type_name()` |
| ----------------- | ------------------ | ------------- |
| `Null`            | —                  | `null`        |
| `Bool(bool)`      | boolean            | `boolean`     |
| `Integer(i64)`    | signed integer     | `integer`     |
| `Float(f64)`      | float              | `float`       |
| `String(String)`  | text               | `string`      |
| `Array(Vec<Value>)` | ordered list     | `array`       |
| `Object(HashMap<String, Value>)` | keyed map | `object` |
| `Function(String)` | function name     | `function`    |
| `Unknown(String)` | complex/unrepresentable type rendered as text | `unknown` |

`Value::type_name()` returns the stable lowercase strings above (used by the
`ValueFilter` type match). `Value::redacted_nested(patterns)` recursively
scrubs secret-like keys inside `Object`/`Array` values (see
[configuration.md](configuration.md) for the redaction settings and
[filtering.md](filtering.md) is unrelated to redaction).

## `ExecutionEvent` variants

Events are grouped in the source into normal-execution, error/exception,
special-detection, and async/concurrency categories.

### Normal execution

- `FunctionEnter { id, name, args: HashMap<String, Value>, location, timestamp }`
- `FunctionExit { id, name, return_value: Option<Value>, duration: Duration, timestamp }`
  (carries **no** location; `location()` returns `unknown()`)
- `VariableDeclaration { id, name, value: Option<Value>, var_type: Option<String>, is_const: bool, location, timestamp }`
- `VariableAssign { id, name, old_value: Option<Value>, new_value: Value, location, timestamp }`
- `ConditionalEval { id, condition: String, result: bool, branch_taken: String, location, timestamp }`
- `LoopEntry { id, loop_type: String, condition: Option<String>, location, timestamp }`
- `LoopIteration { id, loop_type, iteration: usize, loop_var: Option<String>, loop_var_value: Option<Value>, timestamp }`
  (no location)
- `LoopExit { id, loop_type, total_iterations: usize, reason: LoopExitReason, timestamp }`
  (no location)
- `Return { id, value: Option<Value>, location, timestamp }`

`LoopExitReason` is one of `ConditionFalse`, `Break`, `Return`, `Exception`.

### Errors and exceptions

- `Exception { id, error_type, message, location, stack_trace: Vec<StackFrame>, caught: bool, timestamp }`
- `SyntaxError { id, message, location, offending_code, suggestion: Option<String>, timestamp }`
- `RuntimeError { id, error_type, message, location, context: HashMap<String, Value>, stack_trace, timestamp }`
- `TypeError { id, expected, got, value: Value, operation, location, timestamp }`
- `NullPointerError { id, variable, operation, location, timestamp }`
- `IndexOutOfBounds { id, index: i64, size: usize, collection, location, timestamp }`
- `DivisionByZero { id, numerator: Value, denominator_var: Option<String>, location, timestamp }`
- `StackOverflow { id, function, recursion_depth: usize, location, timestamp }`
- `Panic { id, message, location, stack_trace, timestamp }`

`StackFrame { function_name: String, location: SourceLocation, arguments: HashMap<String, Value> }`.

### Special detection

- `InfiniteLoopDetected { id, loop_type, iterations: usize, location, timestamp }`
- `DeadlockDetected { id, threads: Vec<String>, timestamp }` (no location)
- `MemoryLeakDetected { id, allocation_count: usize, leaked_bytes: usize, timestamp }` (no location)

### Async / concurrency

- `AsyncTaskStart { id, task_id: Uuid, task_name: String, spawned_from: SourceLocation, timestamp }`
- `AsyncTaskAwait { id, task_id, awaiting_on: String, location, timestamp }`
- `AsyncTaskResume { id, task_id, resumed_with: Option<Value>, location, timestamp }`

There is currently **no** `AsyncTaskComplete` variant. See
[async.md](async.md) for how completion is handled.

## Accessor methods

- `id() -> &Uuid`
- `timestamp() -> &DateTime<Utc>`
- `location() -> SourceLocation` (clones; returns `unknown()` for locationless variants)
- `location_ref() -> Option<&SourceLocation>` (allocation-free; `None` for locationless variants)
- `event_type() -> &'static str` (stable snake_case tag, e.g. `function_enter`,
  `division_by_zero`, `async_task_resume`)
- `is_error() -> bool` (true for the error and special-detection variants listed above)

## Redaction

- `redacted(&self, patterns: &[String]) -> ExecutionEvent` returns a clone with
  secret-like named values replaced by the placeholder `<redacted>`.
- `redact_events(events, patterns) -> Vec<ExecutionEvent>` maps `redacted` over a slice.

Named-map variants (`FunctionEnter.args`, `RuntimeError.context`, stack-frame
`arguments`) redact any value whose key matches a pattern. `VariableDeclaration`
and `VariableAssign` redact by the variable's own `name`. Single-value payload
fields carry no key of their own, so they are not blanket-redacted, but if the
value is (or contains) an object/array with secret-like keys those inner secrets
are still scrubbed for these fields: `FunctionExit.return_value`, `Return.value`,
`AsyncTaskResume.resumed_with`, `LoopIteration.loop_var_value`,
`TypeError.value`, and `DivisionByZero.numerator`.
