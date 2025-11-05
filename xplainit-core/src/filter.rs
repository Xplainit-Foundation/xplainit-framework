//! Event Filter - Selective event capture
//! 
//! Filters determine which events should be captured and which should be ignored.
//! This allows for precise control over tracing scope and performance.

use crate::{ExecutionEvent, Config};
use std::collections::HashSet;

/// Trait for filtering events
pub trait EventFilter: Send + Sync {
    /// Returns true if the event should be captured
    fn should_capture(&self, event: &ExecutionEvent, config: &Config) -> bool;
    
    /// Returns a description of this filter
    fn description(&self) -> String;
}

/// Filter that accepts all events
#[derive(Debug, Clone, Default)]
pub struct AcceptAllFilter;

impl EventFilter for AcceptAllFilter {
    fn should_capture(&self, _event: &ExecutionEvent, _config: &Config) -> bool {
        true
    }
    
    fn description(&self) -> String {
        "Accept all events".to_string()
    }
}

/// Filter based on function names
#[derive(Debug, Clone)]
pub struct FunctionFilter {
    /// Functions to include (if empty, all functions are included)
    pub include: HashSet<String>,
    /// Functions to exclude
    pub exclude: HashSet<String>,
    /// Whether to trace standard library functions
    pub trace_stdlib: bool,
}

impl FunctionFilter {
    pub fn new() -> Self {
        Self {
            include: HashSet::new(),
            exclude: HashSet::new(),
            trace_stdlib: false,
        }
    }
    
    pub fn include(mut self, func: impl Into<String>) -> Self {
        self.include.insert(func.into());
        self
    }
    
    pub fn exclude(mut self, func: impl Into<String>) -> Self {
        self.exclude.insert(func.into());
        self
    }
    
    pub fn with_stdlib(mut self, trace: bool) -> Self {
        self.trace_stdlib = trace;
        self
    }
}

impl Default for FunctionFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl EventFilter for FunctionFilter {
    fn should_capture(&self, event: &ExecutionEvent, _config: &Config) -> bool {
        // Get function name from event if applicable
        let func_name = match event {
            ExecutionEvent::FunctionEnter { name, .. } => Some(name.as_str()),
            ExecutionEvent::FunctionExit { name, .. } => Some(name.as_str()),
            _ => None,
        };
        
        if let Some(name) = func_name {
            // Check exclusions first
            if self.exclude.contains(name) {
                return false;
            }
            
            // If include list is specified, only include those
            if !self.include.is_empty() && !self.include.contains(name) {
                return false;
            }
            
            // Check stdlib
            if !self.trace_stdlib && is_stdlib_function(name) {
                return false;
            }
        }
        
        true
    }
    
    fn description(&self) -> String {
        format!(
            "Function filter (include: {}, exclude: {}, stdlib: {})",
            self.include.len(),
            self.exclude.len(),
            self.trace_stdlib
        )
    }
}

/// Filter based on event types
#[derive(Debug, Clone, Default)]
pub struct EventTypeFilter {
    /// Event types to capture
    pub capture_normal: bool,
    pub capture_errors: bool,
    pub capture_functions: bool,
    pub capture_variables: bool,
    pub capture_loops: bool,
}

impl EventTypeFilter {
    pub fn new() -> Self {
        Self {
            capture_normal: true,
            capture_errors: true,
            capture_functions: true,
            capture_variables: true,
            capture_loops: true,
        }
    }
    
    pub fn only_errors() -> Self {
        Self {
            capture_normal: false,
            capture_errors: true,
            capture_functions: false,
            capture_variables: false,
            capture_loops: false,
        }
    }
    
    pub fn only_functions() -> Self {
        Self {
            capture_normal: false,
            capture_errors: false,
            capture_functions: true,
            capture_variables: false,
            capture_loops: false,
        }
    }
}

impl EventFilter for EventTypeFilter {
    fn should_capture(&self, event: &ExecutionEvent, _config: &Config) -> bool {
        match event {
            ExecutionEvent::FunctionEnter { .. } | ExecutionEvent::FunctionExit { .. } => {
                self.capture_functions
            }
            ExecutionEvent::VariableDeclaration { .. } | ExecutionEvent::VariableAssign { .. } => {
                self.capture_variables
            }
            ExecutionEvent::LoopEntry { .. } 
            | ExecutionEvent::LoopIteration { .. } 
            | ExecutionEvent::LoopExit { .. } => {
                self.capture_loops
            }
            _ if event.is_error() => {
                self.capture_errors
            }
            _ => {
                self.capture_normal
            }
        }
    }
    
    fn description(&self) -> String {
        format!(
            "Event type filter (functions: {}, variables: {}, loops: {}, errors: {})",
            self.capture_functions,
            self.capture_variables,
            self.capture_loops,
            self.capture_errors
        )
    }
}

/// Filter based on depth (stack depth)
#[derive(Debug, Clone)]
pub struct DepthFilter {
    max_depth: usize,
    current_depth: usize,
}

impl DepthFilter {
    pub fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            current_depth: 0,
        }
    }
}

impl EventFilter for DepthFilter {
    fn should_capture(&self, event: &ExecutionEvent, _config: &Config) -> bool {
        match event {
            ExecutionEvent::FunctionEnter { .. } => {
                self.current_depth < self.max_depth
            }
            _ => true,
        }
    }
    
    fn description(&self) -> String {
        format!("Depth filter (max: {})", self.max_depth)
    }
}

/// Composite filter that combines multiple filters
#[derive(Default)]
pub struct CompositeFilter {
    filters: Vec<Box<dyn EventFilter>>,
    /// If true, ALL filters must pass; if false, ANY filter can pass
    require_all: bool,
}

impl CompositeFilter {
    pub fn new(require_all: bool) -> Self {
        Self {
            filters: Vec::new(),
            require_all,
        }
    }
    
    pub fn add_filter(mut self, filter: Box<dyn EventFilter>) -> Self {
        self.filters.push(filter);
        self
    }
}

impl EventFilter for CompositeFilter {
    fn should_capture(&self, event: &ExecutionEvent, config: &Config) -> bool {
        if self.filters.is_empty() {
            return true;
        }
        
        if self.require_all {
            // ALL filters must pass
            self.filters.iter().all(|f| f.should_capture(event, config))
        } else {
            // ANY filter can pass
            self.filters.iter().any(|f| f.should_capture(event, config))
        }
    }
    
    fn description(&self) -> String {
        format!(
            "Composite filter ({} filters, require_all: {})",
            self.filters.len(),
            self.require_all
        )
    }
}

/// Check if a function name belongs to standard library
fn is_stdlib_function(name: &str) -> bool {
    // Common stdlib patterns across languages
    name.starts_with("std::")
        || name.starts_with("__")
        || name.starts_with("_")
        || name.contains("builtins")
        || name.contains("System.")
        || name.contains("java.lang")
        || name.contains("java.util")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceLocation;
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_accept_all_filter() {
        let filter = AcceptAllFilter;
        let config = Config::new(crate::Language::Python);
        
        let event = ExecutionEvent::FunctionEnter {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "test.py".into(),
                line: 1,
                column: 0,
                offset: 0,
            },
            name: "test".into(),
            args: HashMap::new(),
        };
        
        assert!(filter.should_capture(&event, &config));
    }

    #[test]
    fn test_function_filter_include() {
        let filter = FunctionFilter::new()
            .include("allowed_func");
        
        let config = Config::new(crate::Language::Python);
        
        let allowed = ExecutionEvent::FunctionEnter {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "test.py".into(),
                line: 1,
                column: 0,
                offset: 0,
            },
            name: "allowed_func".into(),
            args: HashMap::new(),
        };
        
        let not_allowed = ExecutionEvent::FunctionEnter {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "test.py".into(),
                line: 2,
                column: 0,
                offset: 0,
            },
            name: "other_func".into(),
            args: HashMap::new(),
        };
        
        assert!(filter.should_capture(&allowed, &config));
        assert!(!filter.should_capture(&not_allowed, &config));
    }

    #[test]
    fn test_function_filter_exclude() {
        let filter = FunctionFilter::new()
            .exclude("blocked_func");
        
        let config = Config::new(crate::Language::Python);
        
        let blocked = ExecutionEvent::FunctionEnter {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "test.py".into(),
                line: 1,
                column: 0,
                offset: 0,
            },
            name: "blocked_func".into(),
            args: HashMap::new(),
        };
        
        assert!(!filter.should_capture(&blocked, &config));
    }

    #[test]
    fn test_event_type_filter() {
        let filter = EventTypeFilter::only_errors();
        let config = Config::new(crate::Language::Python);
        
        let normal = ExecutionEvent::FunctionEnter {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "test.py".into(),
                line: 1,
                column: 0,
                offset: 0,
            },
            name: "test".into(),
            args: HashMap::new(),
        };
        
        let error = ExecutionEvent::RuntimeError {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "test.py".into(),
                line: 2,
                column: 0,
                offset: 0,
            },
            message: "Error".into(),
            error_type: "RuntimeError".into(),
            stack_trace: vec![],
            context: HashMap::new(),
        };
        
        assert!(!filter.should_capture(&normal, &config));
        assert!(filter.should_capture(&error, &config));
    }
}
