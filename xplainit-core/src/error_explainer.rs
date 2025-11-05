//! Error & Exception Explanation System
//! Provides root cause analysis, context, and fix suggestions for errors
use crate::events::*;

/// Error severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Warning - program can continue
    Warning,
    /// Error - program behavior is incorrect but may continue
    Error,
    /// Critical - program will likely crash or produce wrong results
    Critical,
    /// Fatal - program must terminate
    Fatal,
}

/// Error category for grouping similar errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Logic,          // Logic errors (wrong algorithm, incorrect conditions)
    Type,           // Type-related errors
    Memory,         // Memory-related errors (null, out of bounds)
    Arithmetic,     // Math-related errors (division by zero, overflow)
    Concurrency,    // Threading/concurrency issues
    Resource,       // Resource exhaustion (stack overflow, memory leak)
    Syntax,         // Syntax errors
    Runtime,        // General runtime errors
}

/// Detailed error analysis result
#[derive(Debug, Clone)]
pub struct ErrorAnalysis {
    /// The original event
    pub event: ExecutionEvent,
    
    /// Severity of the error
    pub severity: ErrorSeverity,
    
    /// Category of the error
    pub category: ErrorCategory,
    
    /// Root cause explanation
    pub root_cause: String,
    
    /// What led to this error (chain of events)
    pub leading_events: Vec<String>,
    
    /// Immediate fix suggestions
    pub fix_suggestions: Vec<String>,
    
    /// Preventive measures for the future
    pub prevention_tips: Vec<String>,
    
    /// Related documentation/resources
    pub resources: Vec<String>,
    
    /// Similar error patterns the user might encounter
    pub related_errors: Vec<String>,
}

/// The main error explanation system
pub struct ErrorExplainer {
    /// Track recent events for context analysis
    recent_events: Vec<ExecutionEvent>,
    
    /// Maximum events to keep for context
    max_context_events: usize,
}

impl ErrorExplainer {
    pub fn new() -> Self {
        Self {
            recent_events: Vec::new(),
            max_context_events: 100,
        }
    }
    
    pub fn with_context_size(mut self, size: usize) -> Self {
        self.max_context_events = size;
        self
    }
    
    /// Add an event to the context history
    pub fn track_event(&mut self, event: ExecutionEvent) {
        self.recent_events.push(event);
        
        // Keep only the most recent events
        if self.recent_events.len() > self.max_context_events {
            self.recent_events.remove(0);
        }
    }
    
    /// Analyze an error event in depth
    pub fn analyze(&self, event: &ExecutionEvent) -> Option<ErrorAnalysis> {
        if !event.is_error() {
            return None;
        }
        
        match event {
            ExecutionEvent::SyntaxError { .. } => Some(self.analyze_syntax_error(event)),
            ExecutionEvent::TypeError { .. } => Some(self.analyze_type_error(event)),
            ExecutionEvent::NullPointerError { .. } => Some(self.analyze_null_pointer_error(event)),
            ExecutionEvent::IndexOutOfBounds { .. } => Some(self.analyze_index_error(event)),
            ExecutionEvent::DivisionByZero { .. } => Some(self.analyze_division_by_zero(event)),
            ExecutionEvent::StackOverflow { .. } => Some(self.analyze_stack_overflow(event)),
            ExecutionEvent::RuntimeError { .. } => Some(self.analyze_runtime_error(event)),
            ExecutionEvent::Exception { .. } => Some(self.analyze_exception(event)),
            ExecutionEvent::Panic { .. } => Some(self.analyze_panic(event)),
            ExecutionEvent::InfiniteLoopDetected { .. } => Some(self.analyze_infinite_loop(event)),
            ExecutionEvent::DeadlockDetected { .. } => Some(self.analyze_deadlock(event)),
            ExecutionEvent::MemoryLeakDetected { .. } => Some(self.analyze_memory_leak(event)),
            _ => None,
        }
    }
    
    // ===== Specific Error Analyzers =====
    
    fn analyze_syntax_error(&self, event: &ExecutionEvent) -> ErrorAnalysis {
        if let ExecutionEvent::SyntaxError { message, suggestion, .. } = event {
            let mut fixes = vec![];
            if let Some(sug) = suggestion {
                fixes.push(sug.clone());
            }
            
            // Common syntax error patterns
            if message.contains("unexpected") {
                fixes.push("Check for missing or extra punctuation (brackets, quotes, semicolons)".to_string());
            }
            if message.contains("indentation") || message.contains("indent") {
                fixes.push("Make sure your code has consistent indentation (all tabs or all spaces)".to_string());
            }
            if message.contains("EOF") || message.contains("end of file") {
                fixes.push("You might have unclosed brackets, quotes, or parentheses".to_string());
            }
            
            ErrorAnalysis {
                event: event.clone(),
                severity: ErrorSeverity::Fatal,
                category: ErrorCategory::Syntax,
                root_cause: "The code has a syntax error and cannot be parsed by the compiler/interpreter".to_string(),
                leading_events: vec![],
                fix_suggestions: fixes,
                prevention_tips: vec![
                    "Use a code editor with syntax highlighting".to_string(),
                    "Enable linting tools to catch syntax errors early".to_string(),
                    "Format your code with an automatic formatter".to_string(),
                ],
                resources: vec![],
                related_errors: vec![
                    "Missing semicolons".to_string(),
                    "Unclosed brackets or quotes".to_string(),
                    "Invalid character in code".to_string(),
                ],
            }
        } else {
            self.default_analysis(event)
        }
    }
    
    fn analyze_type_error(&self, event: &ExecutionEvent) -> ErrorAnalysis {
        if let ExecutionEvent::TypeError { expected, got, operation, .. } = event {
            let mut fixes = vec![
                format!("Convert the value to {} before using it in this operation", expected),
                format!("Check that the variable contains a {} type value", expected),
            ];
            
            // Specific type conversion suggestions
            let conversion_hint = match (expected.as_str(), got.as_str()) {
                ("string", "integer") | ("string", "float") => "Use str() or string conversion",
                ("integer", "string") => "Use int() or parseInt()",
                ("float", "string") => "Use float() or parseFloat()",
                ("boolean", _) => "Use bool() or check for truthiness",
                _ => "Ensure type compatibility",
            };
            fixes.push(conversion_hint.to_string());
            
            ErrorAnalysis {
                event: event.clone(),
                severity: ErrorSeverity::Error,
                category: ErrorCategory::Type,
                root_cause: format!("Attempted to {} with the wrong data type (got {}, expected {})", 
                    operation, got, expected),
                leading_events: self.find_related_variable_events(),
                fix_suggestions: fixes,
                prevention_tips: vec![
                    "Use type annotations or type hints in your code".to_string(),
                    "Validate data types before operations".to_string(),
                    "Use a type checker tool (mypy for Python, TypeScript for JS)".to_string(),
                ],
                resources: vec![
                    format!("Learn about {} type conversion", expected),
                ],
                related_errors: vec![
                    "AttributeError (accessing wrong type)".to_string(),
                    "ValueError (wrong value for type)".to_string(),
                ],
            }
        } else {
            self.default_analysis(event)
        }
    }
    
    fn analyze_null_pointer_error(&self, event: &ExecutionEvent) -> ErrorAnalysis {
        if let ExecutionEvent::NullPointerError { variable, operation, .. } = event {
            ErrorAnalysis {
                event: event.clone(),
                severity: ErrorSeverity::Critical,
                category: ErrorCategory::Memory,
                root_cause: format!("Tried to {} but the variable '{}' was null/undefined/none", 
                    operation, variable),
                leading_events: self.find_variable_history(variable),
                fix_suggestions: vec![
                    format!("Check if '{}' is null before using: if {} != null {{ ... }}", variable, variable),
                    format!("Initialize '{}' with a default value", variable),
                    "Use optional chaining (?.) or null-coalescing operators".to_string(),
                    format!("Add a guard clause at the function start to validate '{}'", variable),
                ],
                prevention_tips: vec![
                    "Always initialize variables before use".to_string(),
                    "Use null-safe operators provided by your language".to_string(),
                    "Consider using Option/Maybe types instead of null".to_string(),
                    "Add defensive checks for function parameters".to_string(),
                ],
                resources: vec![
                    "Null safety best practices".to_string(),
                    "Optional/Maybe type patterns".to_string(),
                ],
                related_errors: vec![
                    "NullReferenceException".to_string(),
                    "AttributeError: 'NoneType' object".to_string(),
                    "Cannot read property of undefined".to_string(),
                ],
            }
        } else {
            self.default_analysis(event)
        }
    }
    
    fn analyze_index_error(&self, event: &ExecutionEvent) -> ErrorAnalysis {
        if let ExecutionEvent::IndexOutOfBounds { index, size, collection, .. } = event {
            let valid_range = if *size > 0 {
                format!("0 to {}", size - 1)
            } else {
                "none (array is empty)".to_string()
            };
            
            let mut fixes = vec![
                format!("Check that the index is within bounds: 0 <= index < {}", size),
                "Use length checking before accessing: if (index < array.length) { ... }".to_string(),
            ];
            
            if *index < 0 {
                fixes.push("Negative indices might not be supported in this language".to_string());
            } else if *index as usize >= *size {
                fixes.push(format!("Your index ({}) is too large. The collection only has {} elements", index, size));
            }
            
            if *size == 0 {
                fixes.push(format!("The collection '{}' is empty - check why no elements were added", collection));
            }
            
            ErrorAnalysis {
                event: event.clone(),
                severity: ErrorSeverity::Error,
                category: ErrorCategory::Memory,
                root_cause: format!("Attempted to access index {} in '{}' but valid indices are: {}", 
                    index, collection, valid_range),
                leading_events: self.find_related_array_events(collection),
                fix_suggestions: fixes,
                prevention_tips: vec![
                    "Always check array bounds before accessing".to_string(),
                    "Use .get() methods that return Optional instead of direct indexing".to_string(),
                    "Consider using iterators instead of manual indexing".to_string(),
                    "Add assertions or guards for array access".to_string(),
                ],
                resources: vec![
                    "Safe array access patterns".to_string(),
                    "Boundary checking techniques".to_string(),
                ],
                related_errors: vec![
                    "ArrayIndexOutOfBoundsException".to_string(),
                    "IndexError".to_string(),
                    "RangeError".to_string(),
                ],
            }
        } else {
            self.default_analysis(event)
        }
    }
    
    fn analyze_division_by_zero(&self, event: &ExecutionEvent) -> ErrorAnalysis {
        if let ExecutionEvent::DivisionByZero { denominator_var, .. } = event {
            let var_specific_fix = if let Some(var) = denominator_var {
                vec![
                    format!("Add a check: if {} != 0 {{ result = numerator / {} }}", var, var),
                    format!("Investigate why '{}' became zero", var),
                ]
            } else {
                vec!["Check that the denominator is not zero before dividing".to_string()]
            };
            
            ErrorAnalysis {
                event: event.clone(),
                severity: ErrorSeverity::Critical,
                category: ErrorCategory::Arithmetic,
                root_cause: "Division by zero is mathematically undefined and causes a runtime error".to_string(),
                leading_events: if let Some(var) = denominator_var {
                    self.find_variable_history(var)
                } else {
                    vec![]
                },
                fix_suggestions: var_specific_fix,
                prevention_tips: vec![
                    "Always validate denominators before division".to_string(),
                    "Use try-catch blocks around division operations".to_string(),
                    "Add epsilon checks for floating point: if (abs(denominator) > 1e-10)".to_string(),
                    "Consider what should happen when denominator is zero (return default value?)".to_string(),
                ],
                resources: vec![
                    "Safe division patterns".to_string(),
                    "Floating point precision handling".to_string(),
                ],
                related_errors: vec![
                    "ArithmeticException".to_string(),
                    "ZeroDivisionError".to_string(),
                    "Infinity or NaN results".to_string(),
                ],
            }
        } else {
            self.default_analysis(event)
        }
    }
    
    fn analyze_stack_overflow(&self, event: &ExecutionEvent) -> ErrorAnalysis {
        if let ExecutionEvent::StackOverflow { function, recursion_depth, .. } = event {
            ErrorAnalysis {
                event: event.clone(),
                severity: ErrorSeverity::Fatal,
                category: ErrorCategory::Resource,
                root_cause: format!("Function '{}' recursed {} times without a stopping condition", 
                    function, recursion_depth),
                leading_events: self.find_function_call_pattern(function),
                fix_suggestions: vec![
                    format!("Add or fix the base case in '{}' to stop recursion", function),
                    "Ensure the recursive call parameters are progressing toward the base case".to_string(),
                    "Check that the base case condition can actually be reached".to_string(),
                    "Consider using iteration instead of recursion".to_string(),
                    "If recursion is intentional, increase stack size (platform-specific)".to_string(),
                ],
                prevention_tips: vec![
                    "Always define a clear base case for recursive functions".to_string(),
                    "Add depth limiting to recursive functions during development".to_string(),
                    "Test recursive functions with small inputs first".to_string(),
                    "Use tail recursion optimization when available".to_string(),
                ],
                resources: vec![
                    "Recursion patterns and base cases".to_string(),
                    "Tail call optimization".to_string(),
                    "Converting recursion to iteration".to_string(),
                ],
                related_errors: vec![
                    "StackOverflowError".to_string(),
                    "RecursionError: maximum recursion depth exceeded".to_string(),
                    "Segmentation fault (stack too deep)".to_string(),
                ],
            }
        } else {
            self.default_analysis(event)
        }
    }
    
    fn analyze_runtime_error(&self, event: &ExecutionEvent) -> ErrorAnalysis {
        if let ExecutionEvent::RuntimeError { error_type, message, .. } = event {
            ErrorAnalysis {
                event: event.clone(),
                severity: ErrorSeverity::Error,
                category: ErrorCategory::Runtime,
                root_cause: format!("Runtime error of type '{}': {}", error_type, message),
                leading_events: self.find_recent_events(10),
                fix_suggestions: vec![
                    "Check the error message carefully for clues".to_string(),
                    "Review the code at the error location".to_string(),
                    "Add error handling (try-catch) around this operation".to_string(),
                ],
                prevention_tips: vec![
                    "Use error handling for operations that might fail".to_string(),
                    "Validate inputs before processing".to_string(),
                    "Add logging to track program state".to_string(),
                ],
                resources: vec![],
                related_errors: vec![],
            }
        } else {
            self.default_analysis(event)
        }
    }
    
    fn analyze_exception(&self, event: &ExecutionEvent) -> ErrorAnalysis {
        if let ExecutionEvent::Exception { error_type, message, caught, .. } = event {
            let severity = if *caught {
                ErrorSeverity::Warning
            } else {
                ErrorSeverity::Critical
            };
            
            ErrorAnalysis {
                event: event.clone(),
                severity,
                category: ErrorCategory::Runtime,
                root_cause: format!("{} exception: {}", error_type, message),
                leading_events: self.find_recent_events(5),
                fix_suggestions: vec![
                    "Wrap the code in try-catch to handle this exception".to_string(),
                    "Check what conditions cause this exception and prevent them".to_string(),
                    "Add validation before the operation that throws".to_string(),
                ],
                prevention_tips: vec![
                    "Use defensive programming - validate before operating".to_string(),
                    "Handle expected exceptions explicitly".to_string(),
                    "Log exceptions for debugging".to_string(),
                ],
                resources: vec![],
                related_errors: vec![],
            }
        } else {
            self.default_analysis(event)
        }
    }
    
    fn analyze_panic(&self, event: &ExecutionEvent) -> ErrorAnalysis {
        if let ExecutionEvent::Panic { message, .. } = event {
            ErrorAnalysis {
                event: event.clone(),
                severity: ErrorSeverity::Fatal,
                category: ErrorCategory::Runtime,
                root_cause: format!("Program panic: {}", message),
                leading_events: self.find_recent_events(10),
                fix_suggestions: vec![
                    "Review the panic message for the root cause".to_string(),
                    "Check for unwrap() calls that might panic".to_string(),
                    "Use proper error handling instead of panicking".to_string(),
                ],
                prevention_tips: vec![
                    "Avoid unwrap() in production code - use proper error handling".to_string(),
                    "Use Result/Option types and handle errors gracefully".to_string(),
                    "Add validation to prevent panic conditions".to_string(),
                ],
                resources: vec![
                    "Error handling best practices".to_string(),
                    "Result and Option types".to_string(),
                ],
                related_errors: vec![
                    "panic!() macro".to_string(),
                    "unwrap() on None/Err".to_string(),
                    "expect() failures".to_string(),
                ],
            }
        } else {
            self.default_analysis(event)
        }
    }
    
    fn analyze_infinite_loop(&self, event: &ExecutionEvent) -> ErrorAnalysis {
        if let ExecutionEvent::InfiniteLoopDetected { loop_type, iterations, .. } = event {
            ErrorAnalysis {
                event: event.clone(),
                severity: ErrorSeverity::Critical,
                category: ErrorCategory::Logic,
                root_cause: format!("The {} loop has executed {} iterations without exiting", 
                    loop_type, iterations),
                leading_events: self.find_recent_loop_events(),
                fix_suggestions: vec![
                    "Check the loop condition - ensure it can become false".to_string(),
                    "Verify that loop variables are being updated correctly".to_string(),
                    "Add a break statement when the desired condition is met".to_string(),
                    "Review the loop logic to ensure it progresses toward termination".to_string(),
                ],
                prevention_tips: vec![
                    "Always have a clear exit condition for loops".to_string(),
                    "Use for-loops with counters when possible".to_string(),
                    "Add iteration limits during development for testing".to_string(),
                    "Use debuggers to step through loop logic".to_string(),
                ],
                resources: vec![
                    "Loop patterns and anti-patterns".to_string(),
                    "Debugging infinite loops".to_string(),
                ],
                related_errors: vec![
                    "Program hangs/freezes".to_string(),
                    "Timeout errors".to_string(),
                    "High CPU usage".to_string(),
                ],
            }
        } else {
            self.default_analysis(event)
        }
    }
    
    fn analyze_deadlock(&self, event: &ExecutionEvent) -> ErrorAnalysis {
        if let ExecutionEvent::DeadlockDetected { threads, .. } = event {
            ErrorAnalysis {
                event: event.clone(),
                severity: ErrorSeverity::Fatal,
                category: ErrorCategory::Concurrency,
                root_cause: format!("Deadlock between {} threads - each waiting for resources held by others", 
                    threads.len()),
                leading_events: vec![],
                fix_suggestions: vec![
                    "Ensure all threads acquire locks in the same order".to_string(),
                    "Use a timeout when acquiring locks".to_string(),
                    "Consider using higher-level concurrency primitives".to_string(),
                    "Restructure code to reduce lock contention".to_string(),
                ],
                prevention_tips: vec![
                    "Define a global lock ordering and follow it consistently".to_string(),
                    "Minimize the scope of locks".to_string(),
                    "Use lock-free data structures when possible".to_string(),
                    "Avoid nested locking when possible".to_string(),
                ],
                resources: vec![
                    "Deadlock prevention strategies".to_string(),
                    "Lock-free programming".to_string(),
                    "Concurrent programming best practices".to_string(),
                ],
                related_errors: vec![
                    "Thread hangs".to_string(),
                    "Livelock".to_string(),
                    "Race conditions".to_string(),
                ],
            }
        } else {
            self.default_analysis(event)
        }
    }
    
    fn analyze_memory_leak(&self, event: &ExecutionEvent) -> ErrorAnalysis {
        if let ExecutionEvent::MemoryLeakDetected { allocation_count, leaked_bytes, .. } = event {
            ErrorAnalysis {
                event: event.clone(),
                severity: ErrorSeverity::Warning,
                category: ErrorCategory::Memory,
                root_cause: format!("{} allocations ({} bytes) were not freed", 
                    allocation_count, leaked_bytes),
                leading_events: vec![],
                fix_suggestions: vec![
                    "Ensure all allocated resources are properly freed".to_string(),
                    "Use automatic memory management (RAII, smart pointers, GC)".to_string(),
                    "Profile your application to find the leak source".to_string(),
                    "Check for circular references that prevent cleanup".to_string(),
                ],
                prevention_tips: vec![
                    "Use smart pointers (shared_ptr, Arc, etc.) instead of raw pointers".to_string(),
                    "Follow RAII principles - tie resource lifetime to object lifetime".to_string(),
                    "Use memory profiling tools regularly".to_string(),
                    "Implement proper cleanup in destructors/finalizers".to_string(),
                ],
                resources: vec![
                    "Memory leak detection tools".to_string(),
                    "Smart pointer patterns".to_string(),
                    "RAII in practice".to_string(),
                ],
                related_errors: vec![
                    "OutOfMemoryError".to_string(),
                    "Performance degradation over time".to_string(),
                    "Process memory growth".to_string(),
                ],
            }
        } else {
            self.default_analysis(event)
        }
    }
    
    // ===== Context Analysis Helpers =====
    
    fn find_variable_history(&self, var_name: &str) -> Vec<String> {
        self.recent_events
            .iter()
            .filter_map(|e| match e {
                ExecutionEvent::VariableDeclaration { name, .. } if name == var_name => {
                    Some(format!("Variable '{}' was declared", name))
                }
                ExecutionEvent::VariableAssign { name, old_value, new_value, .. } if name == var_name => {
                    Some(format!("'{}' changed from {:?} to {:?}", name, old_value, new_value))
                }
                _ => None,
            })
            .collect()
    }
    
    fn find_related_variable_events(&self) -> Vec<String> {
        self.recent_events
            .iter()
            .rev()
            .take(5)
            .filter_map(|e| match e {
                ExecutionEvent::VariableAssign { name, .. } => {
                    Some(format!("Variable '{}' was modified", name))
                }
                ExecutionEvent::FunctionEnter { name, .. } => {
                    Some(format!("Entered function '{}'", name))
                }
                _ => None,
            })
            .collect()
    }
    
    fn find_related_array_events(&self, array_name: &str) -> Vec<String> {
        self.recent_events
            .iter()
            .filter_map(|e| match e {
                ExecutionEvent::VariableDeclaration { name, .. } if name == array_name => {
                    Some(format!("Array '{}' was created", name))
                }
                ExecutionEvent::VariableAssign { name, .. } if name == array_name => {
                    Some(format!("Array '{}' was modified", name))
                }
                _ => None,
            })
            .collect()
    }
    
    fn find_function_call_pattern(&self, func_name: &str) -> Vec<String> {
        self.recent_events
            .iter()
            .rev()
            .take(10)
            .filter_map(|e| match e {
                ExecutionEvent::FunctionEnter { name, .. } if name == func_name => {
                    Some(format!("Called '{}'", name))
                }
                _ => None,
            })
            .collect()
    }
    
    fn find_recent_loop_events(&self) -> Vec<String> {
        self.recent_events
            .iter()
            .rev()
            .take(5)
            .filter_map(|e| match e {
                ExecutionEvent::LoopEntry { loop_type, .. } => {
                    Some(format!("Entered {} loop", loop_type))
                }
                ExecutionEvent::LoopIteration { iteration, .. } => {
                    Some(format!("Loop iteration {}", iteration))
                }
                _ => None,
            })
            .collect()
    }
    
    fn find_recent_events(&self, count: usize) -> Vec<String> {
        self.recent_events
            .iter()
            .rev()
            .take(count)
            .map(|e| format!("{:?}", e.event_type()))
            .collect()
    }
    
    fn default_analysis(&self, event: &ExecutionEvent) -> ErrorAnalysis {
        ErrorAnalysis {
            event: event.clone(),
            severity: ErrorSeverity::Error,
            category: ErrorCategory::Runtime,
            root_cause: "An error occurred during program execution".to_string(),
            leading_events: vec![],
            fix_suggestions: vec![
                "Review the error details and stack trace".to_string(),
                "Check the documentation for this error type".to_string(),
            ],
            prevention_tips: vec![
                "Add error handling to catch and handle exceptions".to_string(),
                "Validate inputs and state before operations".to_string(),
            ],
            resources: vec![],
            related_errors: vec![],
        }
    }
}

impl Default for ErrorExplainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;
    use std::collections::HashMap;

    #[test]
    fn test_error_explainer_creation() {
        let explainer = ErrorExplainer::new();
        assert_eq!(explainer.max_context_events, 100);
    }

    #[test]
    fn test_track_event() {
        let mut explainer = ErrorExplainer::new();
        let event = ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            args: HashMap::new(),
            location: SourceLocation::unknown(),
            timestamp: Utc::now(),
        };
        
        explainer.track_event(event);
        assert_eq!(explainer.recent_events.len(), 1);
    }

    #[test]
    fn test_analyze_division_by_zero() {
        let explainer = ErrorExplainer::new();
        let event = ExecutionEvent::DivisionByZero {
            id: Uuid::new_v4(),
            numerator: Value::Integer(10),
            denominator_var: Some("x".to_string()),
            location: SourceLocation::new("test.py".to_string(), 10, 5),
            timestamp: Utc::now(),
        };
        
        let analysis = explainer.analyze(&event).unwrap();
        assert_eq!(analysis.severity, ErrorSeverity::Critical);
        assert_eq!(analysis.category, ErrorCategory::Arithmetic);
        assert!(!analysis.fix_suggestions.is_empty());
    }

    #[test]
    fn test_analyze_type_error() {
        let explainer = ErrorExplainer::new();
        let event = ExecutionEvent::TypeError {
            id: Uuid::new_v4(),
            expected: "integer".to_string(),
            got: "string".to_string(),
            value: Value::String("hello".to_string()),
            operation: "add".to_string(),
            location: SourceLocation::new("test.js".to_string(), 20, 10),
            timestamp: Utc::now(),
        };
        
        let analysis = explainer.analyze(&event).unwrap();
        assert_eq!(analysis.category, ErrorCategory::Type);
        assert!(analysis.root_cause.contains("wrong data type"));
    }

    #[test]
    fn test_analyze_stack_overflow() {
        let explainer = ErrorExplainer::new();
        let event = ExecutionEvent::StackOverflow {
            id: Uuid::new_v4(),
            function: "recursive_func".to_string(),
            recursion_depth: 10000,
            location: SourceLocation::new("test.py".to_string(), 5, 1),
            timestamp: Utc::now(),
        };
        
        let analysis = explainer.analyze(&event).unwrap();
        assert_eq!(analysis.severity, ErrorSeverity::Fatal);
        assert_eq!(analysis.category, ErrorCategory::Resource);
        assert!(analysis.fix_suggestions.iter().any(|s| s.contains("base case")));
    }

    #[test]
    fn test_analyze_non_error_returns_none() {
        let explainer = ErrorExplainer::new();
        let event = ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            args: HashMap::new(),
            location: SourceLocation::unknown(),
            timestamp: Utc::now(),
        };
        
        assert!(explainer.analyze(&event).is_none());
    }

    #[test]
    fn test_context_tracking() {
        let mut explainer = ErrorExplainer::with_context_size(ErrorExplainer::new(), 5);
        
        // Add 10 events
        for i in 0..10 {
            let event = ExecutionEvent::VariableAssign {
                id: Uuid::new_v4(),
                name: format!("var{}", i),
                old_value: None,
                new_value: Value::Integer(i),
                location: SourceLocation::unknown(),
                timestamp: Utc::now(),
            };
            explainer.track_event(event);
        }
        
        // Should only keep the last 5
        assert_eq!(explainer.recent_events.len(), 5);
    }
}
