use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// Source code location
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl SourceLocation {
    pub fn new(file: String, line: usize, column: usize) -> Self {
        Self {
            file,
            line,
            column,
            offset: 0,
        }
    }

    pub fn unknown() -> Self {
        Self {
            file: "<unknown>".to_string(),
            line: 0,
            column: 0,
            offset: 0,
        }
    }
}

/// Runtime value representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Function(String), // Function name
    Unknown(String),  // For complex types we can't represent
}

impl Value {
    /// Recursively redact any secret-like keys nested inside object values.
    ///
    /// Used so that a value which is itself an object (for example a captured
    /// struct or dict) still has its inner `password`/`token` fields scrubbed.
    pub(crate) fn redacted_nested(&self, patterns: &[String]) -> Value {
        match self {
            Value::Object(map) => {
                let mut out = HashMap::with_capacity(map.len());
                for (k, v) in map {
                    if crate::security::is_redaction_key(k, patterns) {
                        out.insert(
                            k.clone(),
                            Value::String(crate::security::REDACTED_PLACEHOLDER.to_string()),
                        );
                    } else {
                        out.insert(k.clone(), v.redacted_nested(patterns));
                    }
                }
                Value::Object(out)
            }
            Value::Array(items) => {
                Value::Array(items.iter().map(|v| v.redacted_nested(patterns)).collect())
            }
            other => other.clone(),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Function(_) => "function",
            Value::Unknown(_) => "unknown",
        }
    }
}

/// Stack frame information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub function_name: String,
    pub location: SourceLocation,
    pub arguments: HashMap<String, Value>,
}

/// Why a loop exited
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopExitReason {
    ConditionFalse,
    Break,
    Return,
    Exception,
}

/// Runtime execution event - captures everything that happens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEvent {
    // ===== Normal Execution Events =====
    /// Function entry
    FunctionEnter {
        id: Uuid,
        name: String,
        args: HashMap<String, Value>,
        location: SourceLocation,
        timestamp: DateTime<Utc>,
    },

    /// Function exit
    FunctionExit {
        id: Uuid,
        name: String,
        return_value: Option<Value>,
        duration: Duration,
        timestamp: DateTime<Utc>,
    },

    /// Variable declaration
    VariableDeclaration {
        id: Uuid,
        name: String,
        value: Option<Value>,
        var_type: Option<String>,
        is_const: bool,
        location: SourceLocation,
        timestamp: DateTime<Utc>,
    },

    /// Variable assignment
    VariableAssign {
        id: Uuid,
        name: String,
        old_value: Option<Value>,
        new_value: Value,
        location: SourceLocation,
        timestamp: DateTime<Utc>,
    },

    /// Conditional evaluation (if, else if, etc.)
    ConditionalEval {
        id: Uuid,
        condition: String,
        result: bool,
        branch_taken: String, // "then", "else", etc.
        location: SourceLocation,
        timestamp: DateTime<Utc>,
    },

    /// Loop entry
    LoopEntry {
        id: Uuid,
        loop_type: String, // "for", "while", "do-while"
        condition: Option<String>,
        location: SourceLocation,
        timestamp: DateTime<Utc>,
    },

    /// Loop iteration
    LoopIteration {
        id: Uuid,
        loop_type: String,
        iteration: usize,
        loop_var: Option<String>,
        loop_var_value: Option<Value>,
        timestamp: DateTime<Utc>,
    },

    /// Loop exit
    LoopExit {
        id: Uuid,
        loop_type: String,
        total_iterations: usize,
        reason: LoopExitReason,
        timestamp: DateTime<Utc>,
    },

    /// Return statement
    Return {
        id: Uuid,
        value: Option<Value>,
        location: SourceLocation,
        timestamp: DateTime<Utc>,
    },

    // ===== Error and Exception Events =====
    /// Generic exception/error raised
    Exception {
        id: Uuid,
        error_type: String,
        message: String,
        location: SourceLocation,
        stack_trace: Vec<StackFrame>,
        caught: bool,
        timestamp: DateTime<Utc>,
    },

    /// Syntax error (before execution)
    SyntaxError {
        id: Uuid,
        message: String,
        location: SourceLocation,
        offending_code: String,
        suggestion: Option<String>,
        timestamp: DateTime<Utc>,
    },

    /// Runtime error during execution
    RuntimeError {
        id: Uuid,
        error_type: String,
        message: String,
        location: SourceLocation,
        context: HashMap<String, Value>,
        stack_trace: Vec<StackFrame>,
        timestamp: DateTime<Utc>,
    },

    /// Type error
    TypeError {
        id: Uuid,
        expected: String,
        got: String,
        value: Value,
        operation: String,
        location: SourceLocation,
        timestamp: DateTime<Utc>,
    },

    /// Null/None/undefined access error
    NullPointerError {
        id: Uuid,
        variable: String,
        operation: String, // what were we trying to do
        location: SourceLocation,
        timestamp: DateTime<Utc>,
    },

    /// Index out of bounds
    IndexOutOfBounds {
        id: Uuid,
        index: i64,
        size: usize,
        collection: String,
        location: SourceLocation,
        timestamp: DateTime<Utc>,
    },

    /// Division by zero
    DivisionByZero {
        id: Uuid,
        numerator: Value,
        denominator_var: Option<String>,
        location: SourceLocation,
        timestamp: DateTime<Utc>,
    },

    /// Stack overflow (recursion limit)
    StackOverflow {
        id: Uuid,
        function: String,
        recursion_depth: usize,
        location: SourceLocation,
        timestamp: DateTime<Utc>,
    },

    /// Panic/abort
    Panic {
        id: Uuid,
        message: String,
        location: SourceLocation,
        stack_trace: Vec<StackFrame>,
        timestamp: DateTime<Utc>,
    },

    // ===== Special Detection Events =====
    /// Infinite loop detected
    InfiniteLoopDetected {
        id: Uuid,
        loop_type: String,
        iterations: usize,
        location: SourceLocation,
        timestamp: DateTime<Utc>,
    },

    /// Deadlock detected
    DeadlockDetected {
        id: Uuid,
        threads: Vec<String>,
        timestamp: DateTime<Utc>,
    },

    /// Memory leak detected
    MemoryLeakDetected {
        id: Uuid,
        allocation_count: usize,
        leaked_bytes: usize,
        timestamp: DateTime<Utc>,
    },

    // ===== Async / Concurrency Events =====
    /// An async task was spawned/started
    AsyncTaskStart {
        id: Uuid,
        task_id: Uuid,
        task_name: String,
        spawned_from: SourceLocation,
        timestamp: DateTime<Utc>,
    },

    /// An async task suspended at an await point
    AsyncTaskAwait {
        id: Uuid,
        task_id: Uuid,
        awaiting_on: String,
        location: SourceLocation,
        timestamp: DateTime<Utc>,
    },

    /// An async task resumed after an await point
    AsyncTaskResume {
        id: Uuid,
        task_id: Uuid,
        resumed_with: Option<Value>,
        location: SourceLocation,
        timestamp: DateTime<Utc>,
    },
}

impl ExecutionEvent {
    /// Get the event's unique ID
    pub fn id(&self) -> &Uuid {
        match self {
            ExecutionEvent::FunctionEnter { id, .. } => id,
            ExecutionEvent::FunctionExit { id, .. } => id,
            ExecutionEvent::VariableDeclaration { id, .. } => id,
            ExecutionEvent::VariableAssign { id, .. } => id,
            ExecutionEvent::ConditionalEval { id, .. } => id,
            ExecutionEvent::LoopEntry { id, .. } => id,
            ExecutionEvent::LoopIteration { id, .. } => id,
            ExecutionEvent::LoopExit { id, .. } => id,
            ExecutionEvent::Return { id, .. } => id,
            ExecutionEvent::Exception { id, .. } => id,
            ExecutionEvent::SyntaxError { id, .. } => id,
            ExecutionEvent::RuntimeError { id, .. } => id,
            ExecutionEvent::TypeError { id, .. } => id,
            ExecutionEvent::NullPointerError { id, .. } => id,
            ExecutionEvent::IndexOutOfBounds { id, .. } => id,
            ExecutionEvent::DivisionByZero { id, .. } => id,
            ExecutionEvent::StackOverflow { id, .. } => id,
            ExecutionEvent::Panic { id, .. } => id,
            ExecutionEvent::InfiniteLoopDetected { id, .. } => id,
            ExecutionEvent::DeadlockDetected { id, .. } => id,
            ExecutionEvent::MemoryLeakDetected { id, .. } => id,
            ExecutionEvent::AsyncTaskStart { id, .. } => id,
            ExecutionEvent::AsyncTaskAwait { id, .. } => id,
            ExecutionEvent::AsyncTaskResume { id, .. } => id,
        }
    }

    /// Get the event's timestamp
    pub fn timestamp(&self) -> &DateTime<Utc> {
        match self {
            ExecutionEvent::FunctionEnter { timestamp, .. } => timestamp,
            ExecutionEvent::FunctionExit { timestamp, .. } => timestamp,
            ExecutionEvent::VariableDeclaration { timestamp, .. } => timestamp,
            ExecutionEvent::VariableAssign { timestamp, .. } => timestamp,
            ExecutionEvent::ConditionalEval { timestamp, .. } => timestamp,
            ExecutionEvent::LoopEntry { timestamp, .. } => timestamp,
            ExecutionEvent::LoopIteration { timestamp, .. } => timestamp,
            ExecutionEvent::LoopExit { timestamp, .. } => timestamp,
            ExecutionEvent::Return { timestamp, .. } => timestamp,
            ExecutionEvent::Exception { timestamp, .. } => timestamp,
            ExecutionEvent::SyntaxError { timestamp, .. } => timestamp,
            ExecutionEvent::RuntimeError { timestamp, .. } => timestamp,
            ExecutionEvent::TypeError { timestamp, .. } => timestamp,
            ExecutionEvent::NullPointerError { timestamp, .. } => timestamp,
            ExecutionEvent::IndexOutOfBounds { timestamp, .. } => timestamp,
            ExecutionEvent::DivisionByZero { timestamp, .. } => timestamp,
            ExecutionEvent::StackOverflow { timestamp, .. } => timestamp,
            ExecutionEvent::Panic { timestamp, .. } => timestamp,
            ExecutionEvent::InfiniteLoopDetected { timestamp, .. } => timestamp,
            ExecutionEvent::DeadlockDetected { timestamp, .. } => timestamp,
            ExecutionEvent::MemoryLeakDetected { timestamp, .. } => timestamp,
            ExecutionEvent::AsyncTaskStart { timestamp, .. } => timestamp,
            ExecutionEvent::AsyncTaskAwait { timestamp, .. } => timestamp,
            ExecutionEvent::AsyncTaskResume { timestamp, .. } => timestamp,
        }
    }

    /// Check if this event represents an error
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            ExecutionEvent::Exception { .. }
                | ExecutionEvent::SyntaxError { .. }
                | ExecutionEvent::RuntimeError { .. }
                | ExecutionEvent::TypeError { .. }
                | ExecutionEvent::NullPointerError { .. }
                | ExecutionEvent::IndexOutOfBounds { .. }
                | ExecutionEvent::DivisionByZero { .. }
                | ExecutionEvent::StackOverflow { .. }
                | ExecutionEvent::Panic { .. }
                | ExecutionEvent::InfiniteLoopDetected { .. }
                | ExecutionEvent::DeadlockDetected { .. }
                | ExecutionEvent::MemoryLeakDetected { .. }
        )
    }

    /// Get a borrowed reference to the event's source location, if it carries
    /// one.
    ///
    /// This is the allocation-free counterpart to [`location`](Self::location):
    /// hot paths that only need to read the file/line (filters, analysis)
    /// should prefer this so they avoid cloning the whole [`SourceLocation`]
    /// (which owns a `String` file path) on every event. Variants that do not
    /// carry a location (for example `FunctionExit`, `LoopIteration`) return
    /// `None` here, matching the `SourceLocation::unknown()` fallback that
    /// [`location`](Self::location) returns for them.
    pub fn location_ref(&self) -> Option<&SourceLocation> {
        match self {
            ExecutionEvent::FunctionEnter { location, .. } => Some(location),
            ExecutionEvent::FunctionExit { .. } => None,
            ExecutionEvent::VariableDeclaration { location, .. } => Some(location),
            ExecutionEvent::VariableAssign { location, .. } => Some(location),
            ExecutionEvent::ConditionalEval { location, .. } => Some(location),
            ExecutionEvent::LoopEntry { location, .. } => Some(location),
            ExecutionEvent::LoopIteration { .. } => None,
            ExecutionEvent::LoopExit { .. } => None,
            ExecutionEvent::Return { location, .. } => Some(location),
            ExecutionEvent::Exception { location, .. } => Some(location),
            ExecutionEvent::SyntaxError { location, .. } => Some(location),
            ExecutionEvent::RuntimeError { location, .. } => Some(location),
            ExecutionEvent::TypeError { location, .. } => Some(location),
            ExecutionEvent::NullPointerError { location, .. } => Some(location),
            ExecutionEvent::IndexOutOfBounds { location, .. } => Some(location),
            ExecutionEvent::DivisionByZero { location, .. } => Some(location),
            ExecutionEvent::StackOverflow { location, .. } => Some(location),
            ExecutionEvent::Panic { location, .. } => Some(location),
            ExecutionEvent::InfiniteLoopDetected { location, .. } => Some(location),
            ExecutionEvent::DeadlockDetected { .. } => None,
            ExecutionEvent::MemoryLeakDetected { .. } => None,
            ExecutionEvent::AsyncTaskStart { spawned_from, .. } => Some(spawned_from),
            ExecutionEvent::AsyncTaskAwait { location, .. } => Some(location),
            ExecutionEvent::AsyncTaskResume { location, .. } => Some(location),
        }
    }

    /// Get the event's source location
    pub fn location(&self) -> SourceLocation {
        match self {
            ExecutionEvent::FunctionEnter { location, .. } => location.clone(),
            ExecutionEvent::FunctionExit { .. } => SourceLocation::unknown(),
            ExecutionEvent::VariableDeclaration { location, .. } => location.clone(),
            ExecutionEvent::VariableAssign { location, .. } => location.clone(),
            ExecutionEvent::ConditionalEval { location, .. } => location.clone(),
            ExecutionEvent::LoopEntry { location, .. } => location.clone(),
            ExecutionEvent::LoopIteration { .. } => SourceLocation::unknown(),
            ExecutionEvent::LoopExit { .. } => SourceLocation::unknown(),
            ExecutionEvent::Return { location, .. } => location.clone(),
            ExecutionEvent::Exception { location, .. } => location.clone(),
            ExecutionEvent::SyntaxError { location, .. } => location.clone(),
            ExecutionEvent::RuntimeError { location, .. } => location.clone(),
            ExecutionEvent::TypeError { location, .. } => location.clone(),
            ExecutionEvent::NullPointerError { location, .. } => location.clone(),
            ExecutionEvent::IndexOutOfBounds { location, .. } => location.clone(),
            ExecutionEvent::DivisionByZero { location, .. } => location.clone(),
            ExecutionEvent::StackOverflow { location, .. } => location.clone(),
            ExecutionEvent::Panic { location, .. } => location.clone(),
            ExecutionEvent::InfiniteLoopDetected { location, .. } => location.clone(),
            ExecutionEvent::DeadlockDetected { .. } => SourceLocation::unknown(),
            ExecutionEvent::MemoryLeakDetected { .. } => SourceLocation::unknown(),
            ExecutionEvent::AsyncTaskStart { spawned_from, .. } => spawned_from.clone(),
            ExecutionEvent::AsyncTaskAwait { location, .. } => location.clone(),
            ExecutionEvent::AsyncTaskResume { location, .. } => location.clone(),
        }
    }

    /// Get event type name
    pub fn event_type(&self) -> &'static str {
        match self {
            ExecutionEvent::FunctionEnter { .. } => "function_enter",
            ExecutionEvent::FunctionExit { .. } => "function_exit",
            ExecutionEvent::VariableDeclaration { .. } => "variable_declaration",
            ExecutionEvent::VariableAssign { .. } => "variable_assign",
            ExecutionEvent::ConditionalEval { .. } => "conditional_eval",
            ExecutionEvent::LoopEntry { .. } => "loop_entry",
            ExecutionEvent::LoopIteration { .. } => "loop_iteration",
            ExecutionEvent::LoopExit { .. } => "loop_exit",
            ExecutionEvent::Return { .. } => "return",
            ExecutionEvent::Exception { .. } => "exception",
            ExecutionEvent::SyntaxError { .. } => "syntax_error",
            ExecutionEvent::RuntimeError { .. } => "runtime_error",
            ExecutionEvent::TypeError { .. } => "type_error",
            ExecutionEvent::NullPointerError { .. } => "null_pointer_error",
            ExecutionEvent::IndexOutOfBounds { .. } => "index_out_of_bounds",
            ExecutionEvent::DivisionByZero { .. } => "division_by_zero",
            ExecutionEvent::StackOverflow { .. } => "stack_overflow",
            ExecutionEvent::Panic { .. } => "panic",
            ExecutionEvent::InfiniteLoopDetected { .. } => "infinite_loop",
            ExecutionEvent::DeadlockDetected { .. } => "deadlock",
            ExecutionEvent::MemoryLeakDetected { .. } => "memory_leak",
            ExecutionEvent::AsyncTaskStart { .. } => "async_task_start",
            ExecutionEvent::AsyncTaskAwait { .. } => "async_task_await",
            ExecutionEvent::AsyncTaskResume { .. } => "async_task_resume",
        }
    }

    /// Return a clone of this event with secret-like named values redacted.
    ///
    /// `patterns` are the case-insensitive substrings that mark a key as
    /// secret (see [`crate::security::DEFAULT_REDACTION_PATTERNS`]). Any
    /// argument, variable or context value whose key matches is replaced with
    /// the redacted placeholder, and nested object/array values are scrubbed
    /// recursively. This is the single choke point the CLI, dashboard and
    /// bindings call before events leave the process (console, JSON, HTTP).
    pub fn redacted(&self, patterns: &[String]) -> ExecutionEvent {
        let mut event = self.clone();
        match &mut event {
            ExecutionEvent::FunctionEnter { args, .. } => redact_named_map(args, patterns),
            ExecutionEvent::VariableDeclaration { name, value, .. } => {
                if crate::security::is_redaction_key(name, patterns) {
                    if let Some(v) = value {
                        *v = Value::String(crate::security::REDACTED_PLACEHOLDER.to_string());
                    }
                } else if let Some(v) = value {
                    *v = v.redacted_nested(patterns);
                }
            }
            ExecutionEvent::VariableAssign {
                name,
                old_value,
                new_value,
                ..
            } => {
                if crate::security::is_redaction_key(name, patterns) {
                    *new_value = Value::String(crate::security::REDACTED_PLACEHOLDER.to_string());
                    if let Some(v) = old_value {
                        *v = Value::String(crate::security::REDACTED_PLACEHOLDER.to_string());
                    }
                } else {
                    *new_value = new_value.redacted_nested(patterns);
                    if let Some(v) = old_value {
                        *v = v.redacted_nested(patterns);
                    }
                }
            }
            ExecutionEvent::RuntimeError { context, .. } => redact_named_map(context, patterns),
            ExecutionEvent::Exception { stack_trace, .. }
            | ExecutionEvent::Panic { stack_trace, .. } => {
                for frame in stack_trace.iter_mut() {
                    redact_named_map(&mut frame.arguments, patterns);
                }
            }
            _ => {}
        }
        event
    }
}

/// Redact secret-like keys in a named-value map in place.
fn redact_named_map(map: &mut HashMap<String, Value>, patterns: &[String]) {
    for (key, value) in map.iter_mut() {
        if crate::security::is_redaction_key(key, patterns) {
            *value = Value::String(crate::security::REDACTED_PLACEHOLDER.to_string());
        } else {
            *value = value.redacted_nested(patterns);
        }
    }
}

/// Redact secret-like named values across a slice of events.
///
/// Convenience wrapper used by the CLI and dashboard so they redact a whole
/// trace with one call before rendering or serializing it.
pub fn redact_events(events: &[ExecutionEvent], patterns: &[String]) -> Vec<ExecutionEvent> {
    events.iter().map(|e| e.redacted(patterns)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_name() {
        assert_eq!(Value::Null.type_name(), "null");
        assert_eq!(Value::Bool(true).type_name(), "boolean");
        assert_eq!(Value::Integer(42).type_name(), "integer");
        assert_eq!(Value::String("test".to_string()).type_name(), "string");
    }

    #[test]
    fn test_event_is_error() {
        let normal_event = ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            args: HashMap::new(),
            location: SourceLocation::unknown(),
            timestamp: Utc::now(),
        };

        assert!(!normal_event.is_error());

        let error_event = ExecutionEvent::DivisionByZero {
            id: Uuid::new_v4(),
            numerator: Value::Integer(10),
            denominator_var: Some("x".to_string()),
            location: SourceLocation::unknown(),
            timestamp: Utc::now(),
        };

        assert!(error_event.is_error());
    }

    #[test]
    fn test_async_task_start_accessors() {
        let id = Uuid::new_v4();
        let task_id = Uuid::new_v4();
        let ts = Utc::now();
        let event = ExecutionEvent::AsyncTaskStart {
            id,
            task_id,
            task_name: "worker".to_string(),
            spawned_from: SourceLocation::new("main.rs".to_string(), 12, 3),
            timestamp: ts,
        };

        assert_eq!(event.id(), &id);
        assert_eq!(event.timestamp(), &ts);
        assert_eq!(event.event_type(), "async_task_start");
        assert_eq!(event.location().file, "main.rs");
        assert_eq!(event.location().line, 12);
        assert!(!event.is_error());
    }

    #[test]
    fn test_async_task_await_accessors() {
        let id = Uuid::new_v4();
        let task_id = Uuid::new_v4();
        let ts = Utc::now();
        let event = ExecutionEvent::AsyncTaskAwait {
            id,
            task_id,
            awaiting_on: "fetch_data".to_string(),
            location: SourceLocation::new("worker.rs".to_string(), 20, 5),
            timestamp: ts,
        };

        assert_eq!(event.id(), &id);
        assert_eq!(event.timestamp(), &ts);
        assert_eq!(event.event_type(), "async_task_await");
        assert_eq!(event.location().file, "worker.rs");
        assert_eq!(event.location().line, 20);
        assert!(!event.is_error());
    }

    #[test]
    fn test_async_task_resume_accessors() {
        let id = Uuid::new_v4();
        let task_id = Uuid::new_v4();
        let ts = Utc::now();
        let event = ExecutionEvent::AsyncTaskResume {
            id,
            task_id,
            resumed_with: Some(Value::Integer(42)),
            location: SourceLocation::new("worker.rs".to_string(), 21, 5),
            timestamp: ts,
        };

        assert_eq!(event.id(), &id);
        assert_eq!(event.timestamp(), &ts);
        assert_eq!(event.event_type(), "async_task_resume");
        assert_eq!(event.location().file, "worker.rs");
        assert_eq!(event.location().line, 21);
        assert!(!event.is_error());
    }

    #[test]
    fn test_async_events_serde_round_trip() {
        let events = vec![
            ExecutionEvent::AsyncTaskStart {
                id: Uuid::new_v4(),
                task_id: Uuid::new_v4(),
                task_name: "worker".to_string(),
                spawned_from: SourceLocation::new("main.rs".to_string(), 12, 3),
                timestamp: Utc::now(),
            },
            ExecutionEvent::AsyncTaskAwait {
                id: Uuid::new_v4(),
                task_id: Uuid::new_v4(),
                awaiting_on: "fetch_data".to_string(),
                location: SourceLocation::new("worker.rs".to_string(), 20, 5),
                timestamp: Utc::now(),
            },
            ExecutionEvent::AsyncTaskResume {
                id: Uuid::new_v4(),
                task_id: Uuid::new_v4(),
                resumed_with: Some(Value::String("done".to_string())),
                location: SourceLocation::new("worker.rs".to_string(), 21, 5),
                timestamp: Utc::now(),
            },
        ];

        for event in events {
            let json = serde_json::to_string(&event).expect("serialize");
            let decoded: ExecutionEvent = serde_json::from_str(&json).expect("deserialize");
            // Compare via the serialized form since ExecutionEvent has no PartialEq.
            let rejson = serde_json::to_string(&decoded).expect("reserialize");
            assert_eq!(json, rejson);
            assert_eq!(event.event_type(), decoded.event_type());
            assert_eq!(event.id(), decoded.id());
        }
    }
}
