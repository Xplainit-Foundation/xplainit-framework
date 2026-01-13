//! Python tracer implementation using sys.settrace()
//! 
//! Provides the Rust backend for Python's sys.settrace() integration

use xplainit_core::*;
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;

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
        events.last()
            .map(|e| generator.explain(e))
            .unwrap_or_else(|| "No explanation available".to_string())
    }
    
    pub fn get_stats(&self) -> String {
        let total = self.engine.event_store().len();
        format!("Events captured: {}, Enabled: {}", total, self.enabled)
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
    let unquoted = if (trimmed.starts_with('"') && trimmed.ends_with('"')) ||
                       (trimmed.starts_with('\'') && trimmed.ends_with('\'')) {
        &trimmed[1..trimmed.len()-1]
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
        assert_eq!(parse_python_value("3.14"), Value::Float(3.14));
        assert_eq!(parse_python_value("'hello'"), Value::String("hello".to_string()));
        assert_eq!(parse_python_value("\"world\""), Value::String("world".to_string()));
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
}
