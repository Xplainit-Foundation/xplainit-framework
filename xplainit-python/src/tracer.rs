//! Python tracer implementation using sys.settrace()
//!
//! Provides the Rust backend for Python's sys.settrace() integration

use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;
use xplainit_core::*;

pub struct PythonTracer {
    engine: RuntimeEngine,
    config: Config,
    enabled: bool,
}

impl PythonTracer {
    pub fn new(config: Config, enabled: bool) -> Self {
        let engine = RuntimeEngine::new(config.clone());

        Self {
            engine,
            config,
            enabled,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn clear(&mut self) {
        self.engine.event_store().clear();
    }

    pub fn set_verbosity(&mut self, level: &str) {
        let verb = match level.to_lowercase().as_str() {
            "brief" => Verbosity::Brief,
            "normal" => Verbosity::Normal,
            "detailed" => Verbosity::Detailed,
            "debug" => Verbosity::Debug,
            _ => Verbosity::Normal,
        };
        self.config.verbosity = verb;
    }

    pub fn get_events_json(&self) -> String {
        let events = self.engine.event_store().snapshot();
        serde_json::to_string(&events).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn get_last_explanation(&self) -> String {
        let events = self.engine.event_store().snapshot();
        if events.is_empty() {
            return "No events captured yet".to_string();
        }

        let verb_level = match self.config.verbosity {
            Verbosity::Brief => VerbosityLevel::Brief,
            Verbosity::Normal => VerbosityLevel::Normal,
            Verbosity::Detailed => VerbosityLevel::Detailed,
            Verbosity::Debug => VerbosityLevel::Debug,
        };
        let generator = ExplanationGenerator::new(verb_level);
        events
            .last()
            .map(|e| generator.explain(e))
            .unwrap_or_else(|| "No explanation available".to_string())
    }

    pub fn get_stats(&self) -> String {
        let total = self.engine.event_store().len();
        format!("Events captured: {}, Enabled: {}", total, self.enabled)
    }

    /// Get natural language explanations for all captured events
    pub fn get_explanations(&self, verbosity: Option<&str>) -> String {
        let events = self.engine.event_store().snapshot();
        if events.is_empty() {
            return "No events captured yet".to_string();
        }

        // Determine verbosity level
        let verb_level = match verbosity.unwrap_or("normal").to_lowercase().as_str() {
            "brief" => VerbosityLevel::Brief,
            "detailed" => VerbosityLevel::Detailed,
            "debug" => VerbosityLevel::Debug,
            _ => VerbosityLevel::Normal,
        };

        let generator = ExplanationGenerator::new(verb_level);

        // Generate explanations for all events
        let explanations: Vec<String> = events.iter().map(|e| generator.explain(e)).collect();

        explanations.join("\n")
    }

    /// Record a function enter event from Python tracer
    pub fn record_function_enter(
        &mut self,
        name: String,
        args: HashMap<String, Value>,
        filename: String,
        line: usize,
    ) {
        if !self.enabled {
            return;
        }

        let event = ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name,
            args,
            location: SourceLocation::new(filename, line, 0),
            timestamp: Utc::now(),
        };

        self.engine.event_store().record(event);
    }

    /// Record a function exit event from Python tracer
    pub fn record_function_exit(
        &mut self,
        name: String,
        return_value: Option<Value>,
        _filename: String,
        _line: usize,
    ) {
        if !self.enabled {
            return;
        }

        let event = ExecutionEvent::FunctionExit {
            id: Uuid::new_v4(),
            name,
            return_value,
            duration: std::time::Duration::from_micros(0),
            timestamp: Utc::now(),
        };

        self.engine.event_store().record(event);
    }

    /// Record an exception event from Python tracer
    pub fn record_exception(
        &mut self,
        exc_type: String,
        exc_message: String,
        filename: String,
        line: usize,
    ) {
        if !self.enabled {
            return;
        }

        let event = ExecutionEvent::Exception {
            id: Uuid::new_v4(),
            error_type: exc_type,
            message: exc_message,
            stack_trace: vec![],
            location: SourceLocation::new(filename, line, 0),
            caught: false,
            timestamp: Utc::now(),
        };

        self.engine.event_store().record(event);
    }
}

/// Parse a Python value string into a Value enum
pub fn parse_python_value(s: &str) -> Value {
    let trimmed = s.trim();

    // None
    if trimmed == "None" {
        return Value::Null;
    }

    // Boolean
    if trimmed == "True" {
        return Value::Bool(true);
    }
    if trimmed == "False" {
        return Value::Bool(false);
    }

    // Integer
    if let Ok(i) = trimmed.parse::<i64>() {
        return Value::Integer(i);
    }

    // Float
    if let Ok(f) = trimmed.parse::<f64>() {
        return Value::Float(f);
    }

    // String (remove quotes if present)
    let unquoted = if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

    Value::String(unquoted.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_python_value() {
        assert_eq!(parse_python_value("None"), Value::Null);
        assert_eq!(parse_python_value("True"), Value::Bool(true));
        assert_eq!(parse_python_value("False"), Value::Bool(false));
        assert_eq!(parse_python_value("42"), Value::Integer(42));
        assert_eq!(parse_python_value("2.5"), Value::Float(2.5));
        assert_eq!(
            parse_python_value("'hello'"),
            Value::String("hello".to_string())
        );
        assert_eq!(
            parse_python_value("\"world\""),
            Value::String("world".to_string())
        );
    }

    #[test]
    fn test_tracer_creation() {
        let config = Config::new(Language::Python);
        let tracer = PythonTracer::new(config, false);
        assert!(!tracer.is_enabled());
    }

    #[test]
    fn test_tracer_enable_disable() {
        let config = Config::new(Language::Python);
        let mut tracer = PythonTracer::new(config, false);

        assert!(!tracer.is_enabled());
        tracer.enable();
        assert!(tracer.is_enabled());
        tracer.disable();
        assert!(!tracer.is_enabled());
    }

    #[test]
    fn test_record_events_flow_through_to_store() {
        // Exercise the real recording code path: an enabled tracer should
        // record a FunctionEnter and FunctionExit into the event store and
        // surface them through get_events_json()/get_stats(). This test would
        // fail if record_function_enter/exit stopped persisting events.
        let config = Config::new(Language::Python);
        let mut tracer = PythonTracer::new(config, true);

        // Nothing recorded yet.
        assert_eq!(tracer.get_events_json(), "[]");
        assert!(tracer.get_stats().contains("Events captured: 0"));

        let mut args = HashMap::new();
        args.insert("a".to_string(), Value::Integer(5));
        args.insert("b".to_string(), Value::Integer(3));
        tracer.record_function_enter("add".to_string(), args, "test.py".to_string(), 10);
        tracer.record_function_exit(
            "add".to_string(),
            Some(Value::Integer(8)),
            "test.py".to_string(),
            12,
        );

        // Both events must be reflected by the store-backed accessors.
        let events_json = tracer.get_events_json();
        assert_ne!(events_json, "[]");
        assert!(events_json.contains("add"));

        let events = serde_json::from_str::<serde_json::Value>(&events_json)
            .expect("events JSON should parse");
        let count = events.as_array().map(|a| a.len()).unwrap_or(0);
        assert!(count > 0, "expected recorded events, got {}", count);
        assert_eq!(count, 2, "expected exactly one enter + one exit event");

        assert!(tracer.get_stats().contains("Events captured: 2"));
    }

    #[test]
    fn test_disabled_tracer_records_nothing() {
        // Complements the recording test: a disabled tracer must NOT record,
        // proving the recorded events above come from the real code path.
        let config = Config::new(Language::Python);
        let mut tracer = PythonTracer::new(config, false);

        tracer.record_function_enter("add".to_string(), HashMap::new(), "test.py".to_string(), 10);

        assert_eq!(tracer.get_events_json(), "[]");
        assert!(tracer.get_stats().contains("Events captured: 0"));
    }
}
