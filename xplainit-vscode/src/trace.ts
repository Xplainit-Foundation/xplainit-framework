// Trace data model for the Xplainit VS Code extension.
//
// These interfaces mirror the JSON that xplainit-core emits and the `xplainit`
// CLI reads. The Rust `ExecutionEvent` enum uses serde's default
// externally-tagged representation, so each event serializes as a single-key
// object whose key is the variant name, for example:
//
//   { "FunctionEnter": { "id": "...", "name": "divide", "args": {}, ... } }
//   { "DivisionByZero": { "id": "...", "numerator": { "Integer": 10 }, ... } }
//
// A trace file is a JSON array of such objects. The Rust `Value` enum is also
// externally tagged, for example { "Integer": 10 }, { "String": "hi" },
// { "Null": null } is represented simply as "Null" (unit variant). We model the
// value loosely as `unknown` and format it defensively.

/// Source location as serialized by xplainit-core `SourceLocation`.
export interface SourceLocation {
  file: string;
  line: number;
  column: number;
  offset: number;
}

/// The set of variant names produced by `ExecutionEvent`. This matches the keys
/// used in the externally-tagged JSON, not the snake_case `event_type()`
/// strings (which are also listed below for reference).
export type EventVariant =
  | "FunctionEnter"
  | "FunctionExit"
  | "VariableDeclaration"
  | "VariableAssign"
  | "ConditionalEval"
  | "LoopEntry"
  | "LoopIteration"
  | "LoopExit"
  | "Return"
  | "Exception"
  | "SyntaxError"
  | "RuntimeError"
  | "TypeError"
  | "NullPointerError"
  | "IndexOutOfBounds"
  | "DivisionByZero"
  | "StackOverflow"
  | "Panic"
  | "InfiniteLoopDetected"
  | "DeadlockDetected"
  | "MemoryLeakDetected";

/// The inner payload of any event. Fields are optional because they differ per
/// variant; only the fields relevant to a given variant will be present. This
/// mirrors the union of every variant's fields in events.rs.
export interface EventPayload {
  id?: string;
  timestamp?: string;
  location?: SourceLocation;
  name?: string;
  args?: Record<string, unknown>;
  return_value?: unknown;
  duration?: unknown;
  value?: unknown;
  old_value?: unknown;
  new_value?: unknown;
  var_type?: string | null;
  is_const?: boolean;
  condition?: string | null;
  result?: boolean;
  branch_taken?: string;
  loop_type?: string;
  iteration?: number;
  loop_var?: string | null;
  loop_var_value?: unknown;
  total_iterations?: number;
  reason?: string;
  error_type?: string;
  message?: string;
  stack_trace?: unknown[];
  caught?: boolean;
  offending_code?: string;
  suggestion?: string | null;
  context?: Record<string, unknown>;
  expected?: string;
  got?: string;
  operation?: string;
  variable?: string;
  index?: number;
  size?: number;
  collection?: string;
  numerator?: unknown;
  denominator_var?: string | null;
  function?: string;
  recursion_depth?: number;
  iterations?: number;
  threads?: string[];
  allocation_count?: number;
  leaked_bytes?: number;
}

/// A single execution event: exactly one key whose name is the variant.
export type RawEvent = { [K in EventVariant]?: EventPayload };

/// Normalized view of an event that is easier to work with in the extension.
export interface TraceEvent {
  variant: EventVariant;
  payload: EventPayload;
  /// Zero-based position in the trace array (playback order).
  index: number;
}

/// Variant names that `ExecutionEvent::is_error()` treats as errors.
const ERROR_VARIANTS: ReadonlySet<EventVariant> = new Set<EventVariant>([
  "Exception",
  "SyntaxError",
  "RuntimeError",
  "TypeError",
  "NullPointerError",
  "IndexOutOfBounds",
  "DivisionByZero",
  "StackOverflow",
  "Panic",
  "InfiniteLoopDetected",
  "DeadlockDetected",
  "MemoryLeakDetected",
]);

/// snake_case discriminator matching `ExecutionEvent::event_type()`.
const EVENT_TYPE_BY_VARIANT: Record<EventVariant, string> = {
  FunctionEnter: "function_enter",
  FunctionExit: "function_exit",
  VariableDeclaration: "variable_declaration",
  VariableAssign: "variable_assign",
  ConditionalEval: "conditional_eval",
  LoopEntry: "loop_entry",
  LoopIteration: "loop_iteration",
  LoopExit: "loop_exit",
  Return: "return",
  Exception: "exception",
  SyntaxError: "syntax_error",
  RuntimeError: "runtime_error",
  TypeError: "type_error",
  NullPointerError: "null_pointer_error",
  IndexOutOfBounds: "index_out_of_bounds",
  DivisionByZero: "division_by_zero",
  StackOverflow: "stack_overflow",
  Panic: "panic",
  InfiniteLoopDetected: "infinite_loop",
  DeadlockDetected: "deadlock",
  MemoryLeakDetected: "memory_leak",
};

export function isErrorVariant(variant: EventVariant): boolean {
  return ERROR_VARIANTS.has(variant);
}

export function eventType(variant: EventVariant): string {
  return EVENT_TYPE_BY_VARIANT[variant] ?? variant;
}

/// Parse a raw JSON array of externally-tagged events into normalized events.
/// Unknown or malformed entries are skipped rather than throwing so that a
/// single bad record does not break the whole trace.
export function parseTrace(json: string): TraceEvent[] {
  const data = JSON.parse(json) as unknown;
  if (!Array.isArray(data)) {
    throw new Error("Trace file must contain a JSON array of events.");
  }

  const events: TraceEvent[] = [];
  data.forEach((raw, index) => {
    const normalized = normalizeEvent(raw, index);
    if (normalized) {
      events.push(normalized);
    }
  });
  return events;
}

function normalizeEvent(raw: unknown, index: number): TraceEvent | undefined {
  if (raw === null || typeof raw !== "object") {
    return undefined;
  }
  const entries = Object.entries(raw as Record<string, unknown>);
  if (entries.length !== 1) {
    return undefined;
  }
  const [key, payload] = entries[0];
  const variant = key as EventVariant;
  if (!(variant in EVENT_TYPE_BY_VARIANT)) {
    return undefined;
  }
  const typedPayload =
    payload && typeof payload === "object"
      ? (payload as EventPayload)
      : ({} as EventPayload);
  return { variant, payload: typedPayload, index };
}

/// Render a serialized Rust `Value` into a short, human-readable string.
/// The Value enum is externally tagged, for example { "Integer": 10 } or
/// { "String": "hi" }; the `Null` unit variant serializes as the string "Null".
export function formatValue(value: unknown): string {
  if (value === undefined) {
    return "<none>";
  }
  if (value === null) {
    return "null";
  }
  if (typeof value === "string") {
    // Unit variant such as "Null".
    return value;
  }
  if (typeof value === "object") {
    const entries = Object.entries(value as Record<string, unknown>);
    if (entries.length === 1) {
      const [tag, inner] = entries[0];
      switch (tag) {
        case "Null":
          return "null";
        case "Bool":
        case "Integer":
        case "Float":
          return String(inner);
        case "String":
          return JSON.stringify(inner);
        case "Function":
          return `fn ${String(inner)}`;
        case "Array":
          return Array.isArray(inner)
            ? `[${inner.map(formatValue).join(", ")}]`
            : "[]";
        case "Object":
          return "{ ... }";
        case "Unknown":
          return `<${String(inner)}>`;
        default:
          return `${tag}(${JSON.stringify(inner)})`;
      }
    }
  }
  return JSON.stringify(value);
}
