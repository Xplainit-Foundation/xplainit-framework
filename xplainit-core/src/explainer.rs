//! Natural Language Explanation Generator
//! Converts ExecutionEvents into human-readable English explanations
use crate::config::Config;
use crate::events::*;
use std::fmt::Write;

/// Explanation verbosity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbosityLevel {
    /// Minimal output - just the key facts
    Brief,
    /// Balanced output - what happened and why
    Normal,
    /// Detailed output - full context with values
    Detailed,
    /// Everything - debugging-level information
    Debug,
}

impl VerbosityLevel {
    pub fn from_config(_config: &Config) -> Self {
        // This will use config.output_format once we map formats to verbosity
        VerbosityLevel::Normal
    }
}

/// The main explanation generator
pub struct ExplanationGenerator {
    verbosity: VerbosityLevel,
    include_timestamps: bool,
    include_ids: bool,
    use_color: bool,
}

impl ExplanationGenerator {
    pub fn new(verbosity: VerbosityLevel) -> Self {
        Self {
            verbosity,
            include_timestamps: false,
            include_ids: false,
            use_color: true,
        }
    }
    
    pub fn with_timestamps(mut self, include: bool) -> Self {
        self.include_timestamps = include;
        self
    }
    
    pub fn with_ids(mut self, include: bool) -> Self {
        self.include_ids = include;
        self
    }
    
    pub fn with_color(mut self, use_color: bool) -> Self {
        self.use_color = use_color;
        self
    }
    
    /// Generate explanation for an event
    pub fn explain(&self, event: &ExecutionEvent) -> String {
        let mut explanation = String::new();
        
        // Add timestamp prefix if enabled
        if self.include_timestamps {
            let _ = write!(explanation, "[{}] ", self.format_timestamp(event.timestamp()));
        }
        
        // Add ID prefix if enabled
        if self.include_ids {
            let _ = write!(explanation, "[{}] ", event.id());
        }
        
        // Generate the main explanation based on event type
        let main_text = match event {
            ExecutionEvent::FunctionEnter { name, args, location, .. } => {
                self.explain_function_enter(name, args, location)
            }
            ExecutionEvent::FunctionExit { name, return_value, duration, .. } => {
                self.explain_function_exit(name, return_value, duration)
            }
            ExecutionEvent::VariableDeclaration { name, value, var_type, is_const, location, .. } => {
                self.explain_variable_declaration(name, value, var_type, *is_const, location)
            }
            ExecutionEvent::VariableAssign { name, old_value, new_value, location, .. } => {
                self.explain_variable_assign(name, old_value, new_value, location)
            }
            ExecutionEvent::ConditionalEval { condition, result, branch_taken, location, .. } => {
                self.explain_conditional_eval(condition, *result, branch_taken, location)
            }
            ExecutionEvent::LoopEntry { loop_type, condition, location, .. } => {
                self.explain_loop_entry(loop_type, condition, location)
            }
            ExecutionEvent::LoopIteration { loop_type, iteration, loop_var, loop_var_value, .. } => {
                self.explain_loop_iteration(loop_type, *iteration, loop_var, loop_var_value)
            }
            ExecutionEvent::LoopExit { loop_type, total_iterations, reason, .. } => {
                self.explain_loop_exit(loop_type, *total_iterations, reason)
            }
            ExecutionEvent::Return { value, location, .. } => {
                self.explain_return(value, location)
            }
            ExecutionEvent::Exception { error_type, message, location, stack_trace, caught, .. } => {
                self.explain_exception(error_type, message, location, stack_trace, *caught)
            }
            ExecutionEvent::SyntaxError { message, location, offending_code, suggestion, .. } => {
                self.explain_syntax_error(message, location, offending_code, suggestion)
            }
            ExecutionEvent::RuntimeError { error_type, message, location, context, stack_trace, .. } => {
                self.explain_runtime_error(error_type, message, location, context, stack_trace)
            }
            ExecutionEvent::TypeError { expected, got, value, operation, location, .. } => {
                self.explain_type_error(expected, got, value, operation, location)
            }
            ExecutionEvent::NullPointerError { variable, operation, location, .. } => {
                self.explain_null_pointer_error(variable, operation, location)
            }
            ExecutionEvent::IndexOutOfBounds { index, size, collection, location, .. } => {
                self.explain_index_out_of_bounds(*index, *size, collection, location)
            }
            ExecutionEvent::DivisionByZero { numerator, denominator_var, location, .. } => {
                self.explain_division_by_zero(numerator, denominator_var, location)
            }
            ExecutionEvent::StackOverflow { function, recursion_depth, location, .. } => {
                self.explain_stack_overflow(function, *recursion_depth, location)
            }
            ExecutionEvent::Panic { message, location, stack_trace, .. } => {
                self.explain_panic(message, location, stack_trace)
            }
            ExecutionEvent::InfiniteLoopDetected { loop_type, iterations, location, .. } => {
                self.explain_infinite_loop(loop_type, *iterations, location)
            }
            ExecutionEvent::DeadlockDetected { threads, .. } => {
                self.explain_deadlock(threads)
            }
            ExecutionEvent::MemoryLeakDetected { allocation_count, leaked_bytes, .. } => {
                self.explain_memory_leak(*allocation_count, *leaked_bytes)
            }
        };
        
        explanation.push_str(&main_text);
        explanation
    }
    
    // ===== Normal Execution Event Explanations =====
    
    fn explain_function_enter(&self, name: &str, args: &std::collections::HashMap<String, Value>, location: &SourceLocation) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("Calling {}", name)
            }
            VerbosityLevel::Normal => {
                if args.is_empty() {
                    format!("Calling function {} with no arguments", name)
                } else {
                    format!("Calling function {} with {} argument(s)", name, args.len())
                }
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                let mut msg = format!("Calling function {} at {}:{}", name, location.file, location.line);
                if !args.is_empty() {
                    msg.push_str("\n  Arguments:");
                    for (arg_name, arg_value) in args {
                        msg.push_str(&format!("\n    {}: {} = {}", arg_name, arg_value.type_name(), self.format_value(arg_value)));
                    }
                }
                msg
            }
        }
    }
    
    fn explain_function_exit(&self, name: &str, return_value: &Option<Value>, duration: &std::time::Duration) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("{} returned", name)
            }
            VerbosityLevel::Normal => {
                if let Some(val) = return_value {
                    format!("Function {} returned {}", name, self.format_value(val))
                } else {
                    format!("Function {} returned (void)", name)
                }
            }
            VerbosityLevel::Detailed => {
                let ret_msg = if let Some(val) = return_value {
                    format!("{} ({})", self.format_value(val), val.type_name())
                } else {
                    "void".to_string()
                };
                format!("Function {} returned {} after {:?}", name, ret_msg, duration)
            }
            VerbosityLevel::Debug => {
                let ret_msg = if let Some(val) = return_value {
                    format!("{} ({})", self.format_value(val), val.type_name())
                } else {
                    "void".to_string()
                };
                format!("Function {} completed and returned {} (execution time: {:.3}ms)", 
                    name, ret_msg, duration.as_secs_f64() * 1000.0)
            }
        }
    }
    
    fn explain_variable_declaration(&self, name: &str, value: &Option<Value>, var_type: &Option<String>, is_const: bool, location: &SourceLocation) -> String {
        let keyword = if is_const { "constant" } else { "variable" };
        
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("Declared {} {}", keyword, name)
            }
            VerbosityLevel::Normal => {
                if let Some(val) = value {
                    format!("Declared {} {} with value {}", keyword, name, self.format_value(val))
                } else {
                    format!("Declared {} {} (uninitialized)", keyword, name)
                }
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                let mut msg = format!("Declared {} {} at {}:{}", keyword, name, location.file, location.line);
                if let Some(t) = var_type {
                    msg.push_str(&format!(" with type {}", t));
                }
                if let Some(val) = value {
                    msg.push_str(&format!("\n  Initial value: {} = {}", val.type_name(), self.format_value(val)));
                }
                msg
            }
        }
    }
    
    fn explain_variable_assign(&self, name: &str, old_value: &Option<Value>, new_value: &Value, location: &SourceLocation) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("{} = {}", name, self.format_value(new_value))
            }
            VerbosityLevel::Normal => {
                format!("Variable {} was assigned the value {}", name, self.format_value(new_value))
            }
            VerbosityLevel::Detailed => {
                if let Some(old) = old_value {
                    format!("Variable {} changed from {} to {} at {}:{}", 
                        name, self.format_value(old), self.format_value(new_value), location.file, location.line)
                } else {
                    format!("Variable {} was set to {} at {}:{}", 
                        name, self.format_value(new_value), location.file, location.line)
                }
            }
            VerbosityLevel::Debug => {
                if let Some(old) = old_value {
                    format!("Assignment at {}:{} - Variable {} changed:\n  Old: {} ({})\n  New: {} ({})", 
                        location.file, location.line, name,
                        self.format_value(old), old.type_name(),
                        self.format_value(new_value), new_value.type_name())
                } else {
                    format!("Assignment at {}:{} - Variable {} set to {} ({})", 
                        location.file, location.line, name, 
                        self.format_value(new_value), new_value.type_name())
                }
            }
        }
    }
    
    fn explain_conditional_eval(&self, condition: &str, result: bool, branch_taken: &str, location: &SourceLocation) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("Condition {} -> {}", condition, if result { "true" } else { "false" })
            }
            VerbosityLevel::Normal => {
                format!("Evaluated condition '{}' which was {}, so taking the '{}' branch", 
                    condition, if result { "true" } else { "false" }, branch_taken)
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                format!("Conditional evaluation at {}:{}\n  Condition: {}\n  Result: {}\n  Branch taken: {}", 
                    location.file, location.line, condition, 
                    if result { "true" } else { "false" }, branch_taken)
            }
        }
    }
    
    fn explain_loop_entry(&self, loop_type: &str, condition: &Option<String>, location: &SourceLocation) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("Entering {} loop", loop_type)
            }
            VerbosityLevel::Normal => {
                if let Some(cond) = condition {
                    format!("Starting a {} loop with condition: {}", loop_type, cond)
                } else {
                    format!("Starting a {} loop", loop_type)
                }
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                let mut msg = format!("Entering {} loop at {}:{}", loop_type, location.file, location.line);
                if let Some(cond) = condition {
                    msg.push_str(&format!("\n  Condition: {}", cond));
                }
                msg
            }
        }
    }
    
    fn explain_loop_iteration(&self, loop_type: &str, iteration: usize, loop_var: &Option<String>, loop_var_value: &Option<Value>) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("Loop iteration {}", iteration)
            }
            VerbosityLevel::Normal => {
                if let (Some(var), Some(val)) = (loop_var, loop_var_value) {
                    format!("{} loop iteration {} with {} = {}", loop_type, iteration, var, self.format_value(val))
                } else {
                    format!("{} loop iteration {}", loop_type, iteration)
                }
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                let mut msg = format!("{} loop - iteration #{}", loop_type, iteration);
                if let (Some(var), Some(val)) = (loop_var, loop_var_value) {
                    msg.push_str(&format!("\n  Loop variable: {} = {} ({})", var, self.format_value(val), val.type_name()));
                }
                msg
            }
        }
    }
    
    fn explain_loop_exit(&self, loop_type: &str, total_iterations: usize, reason: &LoopExitReason) -> String {
        let reason_text = match reason {
            LoopExitReason::ConditionFalse => "the condition became false",
            LoopExitReason::Break => "a break statement",
            LoopExitReason::Return => "a return statement",
            LoopExitReason::Exception => "an exception was thrown",
        };
        
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("Loop exited after {} iterations", total_iterations)
            }
            VerbosityLevel::Normal => {
                format!("{} loop completed after {} iterations because {}", 
                    loop_type, total_iterations, reason_text)
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                format!("{} loop finished\n  Total iterations: {}\n  Exit reason: {}", 
                    loop_type, total_iterations, reason_text)
            }
        }
    }
    
    fn explain_return(&self, value: &Option<Value>, location: &SourceLocation) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                if let Some(val) = value {
                    format!("Returning {}", self.format_value(val))
                } else {
                    "Returning".to_string()
                }
            }
            VerbosityLevel::Normal => {
                if let Some(val) = value {
                    format!("Returning value {} from current function", self.format_value(val))
                } else {
                    "Returning from current function (void)".to_string()
                }
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                let val_text = if let Some(val) = value {
                    format!("{} ({})", self.format_value(val), val.type_name())
                } else {
                    "void".to_string()
                };
                format!("Return statement at {}:{}\n  Returning: {}", 
                    location.file, location.line, val_text)
            }
        }
    }
    
    // ===== Error Event Explanations =====
    
    fn explain_exception(&self, error_type: &str, message: &str, location: &SourceLocation, stack_trace: &[StackFrame], caught: bool) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("❌ {}: {}", error_type, message)
            }
            VerbosityLevel::Normal => {
                let status = if caught { "caught" } else { "uncaught" };
                format!("❌ An {} {} exception occurred: {}\n  Location: {}:{}", 
                    status, error_type, message, location.file, location.line)
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                let status = if caught { "caught" } else { "UNCAUGHT" };
                let mut msg = format!("❌ EXCEPTION ({} {})\n", status, error_type);
                msg.push_str(&format!("  Error: {}\n", message));
                msg.push_str(&format!("  Location: {}:{}\n", location.file, location.line));
                if self.verbosity == VerbosityLevel::Debug && !stack_trace.is_empty() {
                    msg.push_str("  Stack trace:\n");
                    for (i, frame) in stack_trace.iter().enumerate() {
                        msg.push_str(&format!("    {} {} at {}:{}\n", 
                            i, frame.function_name, frame.location.file, frame.location.line));
                    }
                }
                msg
            }
        }
    }
    
    fn explain_syntax_error(&self, message: &str, location: &SourceLocation, offending_code: &str, suggestion: &Option<String>) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("❌ Syntax error: {}", message)
            }
            VerbosityLevel::Normal => {
                let mut msg = format!("❌ SYNTAX ERROR at {}:{}\n", location.file, location.line);
                msg.push_str(&format!("  {}\n", message));
                if let Some(sug) = suggestion {
                    msg.push_str(&format!("  Suggestion: {}", sug));
                }
                msg
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                let mut msg = format!("❌ SYNTAX ERROR at {}:{}\n", location.file, location.line);
                msg.push_str(&format!("  Error: {}\n", message));
                msg.push_str(&format!("  Offending code: {}\n", offending_code));
                if let Some(sug) = suggestion {
                    msg.push_str(&format!("  💡 Suggestion: {}\n", sug));
                }
                msg
            }
        }
    }
    
    fn explain_runtime_error(&self, error_type: &str, message: &str, location: &SourceLocation, 
                           _context: &std::collections::HashMap<String, Value>, stack_trace: &[StackFrame]) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("❌ {}: {}", error_type, message)
            }
            VerbosityLevel::Normal => {
                format!("❌ RUNTIME ERROR: {}\n  {}\n  Location: {}:{}", 
                    error_type, message, location.file, location.line)
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                let mut msg = format!("❌ RUNTIME ERROR ({})\n", error_type);
                msg.push_str(&format!("  What went wrong: {}\n", message));
                msg.push_str(&format!("  Location: {}:{}\n", location.file, location.line));
                if self.verbosity == VerbosityLevel::Debug && !stack_trace.is_empty() {
                    msg.push_str("  Call stack:\n");
                    for frame in stack_trace.iter().take(5) {
                        msg.push_str(&format!("    {} at {}:{}\n", 
                            frame.function_name, frame.location.file, frame.location.line));
                    }
                }
                msg
            }
        }
    }
    
    fn explain_type_error(&self, expected: &str, got: &str, value: &Value, operation: &str, location: &SourceLocation) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("❌ Type error: expected {}, got {}", expected, got)
            }
            VerbosityLevel::Normal => {
                format!("❌ TYPE ERROR at {}:{}\n  Operation: {}\n  Expected type: {}\n  Got: {} with value {}", 
                    location.file, location.line, operation, expected, got, self.format_value(value))
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                let mut msg = format!("❌ TYPE MISMATCH ERROR at {}:{}\n", location.file, location.line);
                msg.push_str(&format!("  What happened: Tried to {} but the value had the wrong type\n", operation));
                msg.push_str(&format!("  Expected: {}\n", expected));
                msg.push_str(&format!("  Got: {} with value {}\n", got, self.format_value(value)));
                msg.push_str(&format!("  💡 Fix: Make sure the value is of type {} before using it in this operation", expected));
                msg
            }
        }
    }
    
    fn explain_null_pointer_error(&self, variable: &str, operation: &str, location: &SourceLocation) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("❌ Null pointer: {}", variable)
            }
            VerbosityLevel::Normal => {
                format!("❌ NULL/NONE ERROR at {}:{}\n  Tried to {} but variable '{}' was null/none/undefined", 
                    location.file, location.line, operation, variable)
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                let mut msg = format!("❌ NULL POINTER ERROR at {}:{}\n", location.file, location.line);
                msg.push_str(&format!("  What happened: Attempted to {} using variable '{}'\n", operation, variable));
                msg.push_str(&format!("  Problem: The variable '{}' was null/none/undefined at this point\n", variable));
                msg.push_str("  💡 Fix: Check if the variable is null before using it, or ensure it's initialized properly");
                msg
            }
        }
    }
    
    fn explain_index_out_of_bounds(&self, index: i64, size: usize, collection: &str, location: &SourceLocation) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("❌ Index {} out of bounds (size: {})", index, size)
            }
            VerbosityLevel::Normal => {
                format!("❌ INDEX OUT OF BOUNDS at {}:{}\n  Tried to access index {} in '{}' but it only has {} elements (valid indices: 0-{})", 
                    location.file, location.line, index, collection, size, size.saturating_sub(1))
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                let mut msg = format!("❌ INDEX OUT OF BOUNDS ERROR at {}:{}\n", location.file, location.line);
                msg.push_str(&format!("  What happened: Tried to access element at index {} in collection '{}'\n", index, collection));
                msg.push_str(&format!("  Problem: The collection only has {} elements\n", size));
                msg.push_str(&format!("  Valid indices: 0 to {}\n", size.saturating_sub(1)));
                msg.push_str(&format!("  Your index: {}\n", index));
                msg.push_str("  💡 Fix: Check that your index is within the valid range (0 to length-1)");
                msg
            }
        }
    }
    
    fn explain_division_by_zero(&self, numerator: &Value, denominator_var: &Option<String>, location: &SourceLocation) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                "❌ Division by zero".to_string()
            }
            VerbosityLevel::Normal => {
                let denom_text = if let Some(var) = denominator_var {
                    format!("variable '{}'", var)
                } else {
                    "the denominator".to_string()
                };
                format!("❌ DIVISION BY ZERO at {}:{}\n  Tried to divide {} by zero ({})", 
                    location.file, location.line, self.format_value(numerator), denom_text)
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                let mut msg = format!("❌ DIVISION BY ZERO ERROR at {}:{}\n", location.file, location.line);
                msg.push_str(&format!("  What happened: Attempted to divide {} by zero\n", self.format_value(numerator)));
                if let Some(var) = denominator_var {
                    msg.push_str(&format!("  Problem: The denominator variable '{}' had the value 0\n", var));
                    msg.push_str(&format!("  💡 Fix: Check that '{}' is not zero before dividing", var));
                } else {
                    msg.push_str("  Problem: Cannot divide by zero in mathematics\n");
                    msg.push_str("  💡 Fix: Check that the denominator is not zero before dividing");
                }
                msg
            }
        }
    }
    
    fn explain_stack_overflow(&self, function: &str, recursion_depth: usize, location: &SourceLocation) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("❌ Stack overflow in {} ({} calls)", function, recursion_depth)
            }
            VerbosityLevel::Normal => {
                format!("❌ STACK OVERFLOW at {}:{}\n  Function '{}' called itself recursively {} times\n  The call stack is full!", 
                    location.file, location.line, function, recursion_depth)
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                let mut msg = format!("❌ STACK OVERFLOW ERROR at {}:{}\n", location.file, location.line);
                msg.push_str(&format!("  What happened: Function '{}' called itself recursively {} times\n", function, recursion_depth));
                msg.push_str("  Problem: The function is calling itself too many times without a base case to stop\n");
                msg.push_str("  This usually means:\n");
                msg.push_str("    1. You're missing a base case in your recursive function\n");
                msg.push_str("    2. The base case condition is never becoming true\n");
                msg.push_str("    3. You have an infinite recursion bug\n");
                msg.push_str(&format!("  💡 Fix: Review the base case in '{}' and ensure it can be reached", function));
                msg
            }
        }
    }
    
    fn explain_panic(&self, message: &str, location: &SourceLocation, stack_trace: &[StackFrame]) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("❌ PANIC: {}", message)
            }
            VerbosityLevel::Normal => {
                format!("❌ PANIC at {}:{}\n  {}", location.file, location.line, message)
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                let mut msg = format!("❌ PROGRAM PANIC at {}:{}\n", location.file, location.line);
                msg.push_str(&format!("  Message: {}\n", message));
                if self.verbosity == VerbosityLevel::Debug && !stack_trace.is_empty() {
                    msg.push_str("  Stack trace:\n");
                    for (i, frame) in stack_trace.iter().enumerate() {
                        msg.push_str(&format!("    {} {} at {}:{}\n", 
                            i, frame.function_name, frame.location.file, frame.location.line));
                    }
                }
                msg
            }
        }
    }
    
    fn explain_infinite_loop(&self, loop_type: &str, iterations: usize, location: &SourceLocation) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("⚠️  Infinite loop detected ({} iterations)", iterations)
            }
            VerbosityLevel::Normal => {
                format!("⚠️  INFINITE LOOP DETECTED at {}:{}\n  The {} loop has run {} iterations without exiting", 
                    location.file, location.line, loop_type, iterations)
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                let mut msg = format!("⚠️  INFINITE LOOP DETECTED at {}:{}\n", location.file, location.line);
                msg.push_str(&format!("  What happened: The {} loop has executed {} iterations\n", loop_type, iterations));
                msg.push_str("  Problem: The loop condition never becomes false, or there's no break statement\n");
                msg.push_str("  This usually means:\n");
                msg.push_str("    1. The loop condition always evaluates to true\n");
                msg.push_str("    2. The variables in the condition aren't being updated correctly\n");
                msg.push_str("    3. You forgot to include a break or return statement\n");
                msg.push_str("  💡 Fix: Review the loop condition and ensure it can eventually become false");
                msg
            }
        }
    }
    
    fn explain_deadlock(&self, threads: &[String]) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("❌ Deadlock detected ({} threads)", threads.len())
            }
            VerbosityLevel::Normal => {
                format!("❌ DEADLOCK DETECTED\n  {} threads are waiting for each other: {}", 
                    threads.len(), threads.join(", "))
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                let mut msg = "❌ DEADLOCK DETECTED\n".to_string();
                msg.push_str(&format!("  What happened: {} threads are stuck waiting for each other\n", threads.len()));
                msg.push_str("  Threads involved:\n");
                for thread in threads {
                    msg.push_str(&format!("    - {}\n", thread));
                }
                msg.push_str("  Problem: Each thread is waiting for a resource held by another thread\n");
                msg.push_str("  💡 Fix: Review lock acquisition order and ensure all threads acquire locks in the same sequence");
                msg
            }
        }
    }
    
    fn explain_memory_leak(&self, allocation_count: usize, leaked_bytes: usize) -> String {
        match self.verbosity {
            VerbosityLevel::Brief => {
                format!("⚠️  Memory leak: {} bytes", leaked_bytes)
            }
            VerbosityLevel::Normal => {
                format!("⚠️  MEMORY LEAK DETECTED\n  {} allocations totaling {} bytes were never freed", 
                    allocation_count, leaked_bytes)
            }
            VerbosityLevel::Detailed | VerbosityLevel::Debug => {
                let mut msg = "⚠️  MEMORY LEAK DETECTED\n".to_string();
                msg.push_str("  What happened: Memory was allocated but never released\n");
                msg.push_str(&format!("  Leaked allocations: {}\n", allocation_count));
                msg.push_str(&format!("  Total leaked memory: {} bytes ({:.2} KB)\n", leaked_bytes, leaked_bytes as f64 / 1024.0));
                msg.push_str("  Problem: Objects were created but not properly cleaned up\n");
                msg.push_str("  💡 Fix: Ensure all allocated memory is freed, or use automatic memory management");
                msg
            }
        }
    }
    
    // ===== Helper Methods =====
    
    #[allow(clippy::only_used_in_recursion)]
    fn format_value(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => format!("{:.2}", f),
            Value::String(s) => format!("\"{}\"", s),
            Value::Array(arr) => {
                if arr.len() <= 3 {
                    format!("[{}]", arr.iter().map(|v| self.format_value(v)).collect::<Vec<_>>().join(", "))
                } else {
                    format!("[{} elements]", arr.len())
                }
            }
            Value::Object(obj) => {
                if obj.len() <= 2 {
                    let items: Vec<String> = obj.iter().map(|(k, v)| format!("{}: {}", k, self.format_value(v))).collect();
                    format!("{{{}}}", items.join(", "))
                } else {
                    format!("{{{}  fields}}", obj.len())
                }
            }
            Value::Function(name) => format!("<function {}>", name),
            Value::Unknown(desc) => format!("<{}>", desc),
        }
    }
    
    fn format_timestamp(&self, timestamp: &chrono::DateTime<chrono::Utc>) -> String {
        timestamp.format("%H:%M:%S%.3f").to_string()
    }
}

impl Default for ExplanationGenerator {
    fn default() -> Self {
        Self::new(VerbosityLevel::Normal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;
    use std::collections::HashMap;

    #[test]
    fn test_explain_function_enter() {
        let explainer = ExplanationGenerator::new(VerbosityLevel::Normal);
        let mut args = HashMap::new();
        args.insert("x".to_string(), Value::Integer(42));
        
        let event = ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "calculate".to_string(),
            args,
            location: SourceLocation::new("test.py".to_string(), 10, 5),
            timestamp: Utc::now(),
        };
        
        let explanation = explainer.explain(&event);
        assert!(explanation.contains("Calling function calculate"));
        assert!(explanation.contains("1 argument"));
    }
    
    #[test]
    fn test_explain_division_by_zero() {
        let explainer = ExplanationGenerator::new(VerbosityLevel::Normal);
        
        let event = ExecutionEvent::DivisionByZero {
            id: Uuid::new_v4(),
            numerator: Value::Integer(10),
            denominator_var: Some("divisor".to_string()),
            location: SourceLocation::new("math.py".to_string(), 42, 10),
            timestamp: Utc::now(),
        };
        
        let explanation = explainer.explain(&event);
        assert!(explanation.contains("❌"));
        assert!(explanation.contains("DIVISION BY ZERO"));
        assert!(explanation.contains("10"));
    }
    
    #[test]
    fn test_explain_type_error() {
        let explainer = ExplanationGenerator::new(VerbosityLevel::Detailed);
        
        let event = ExecutionEvent::TypeError {
            id: Uuid::new_v4(),
            expected: "integer".to_string(),
            got: "string".to_string(),
            value: Value::String("hello".to_string()),
            operation: "add numbers".to_string(),
            location: SourceLocation::new("app.js".to_string(), 15, 20),
            timestamp: Utc::now(),
        };
        
        let explanation = explainer.explain(&event);
        assert!(explanation.contains("TYPE MISMATCH"));
        assert!(explanation.contains("Expected: integer"));
        assert!(explanation.contains("Got: string"));
        assert!(explanation.contains("💡"));
    }
    
    #[test]
    fn test_verbosity_brief() {
        let explainer = ExplanationGenerator::new(VerbosityLevel::Brief);
        
        let event = ExecutionEvent::VariableAssign {
            id: Uuid::new_v4(),
            name: "count".to_string(),
            old_value: Some(Value::Integer(0)),
            new_value: Value::Integer(1),
            location: SourceLocation::unknown(),
            timestamp: Utc::now(),
        };
        
        let explanation = explainer.explain(&event);
        assert!(explanation.contains("count"));
        assert!(explanation.contains("1"));
        // Brief mode should be short
        assert!(explanation.len() < 50);
    }
    
    #[test]
    fn test_explain_with_timestamps() {
        let explainer = ExplanationGenerator::new(VerbosityLevel::Normal).with_timestamps(true);
        
        let event = ExecutionEvent::Return {
            id: Uuid::new_v4(),
            value: Some(Value::Bool(true)),
            location: SourceLocation::unknown(),
            timestamp: Utc::now(),
        };
        
        let explanation = explainer.explain(&event);
        assert!(explanation.starts_with("["));
        assert!(explanation.contains(":"));
    }
}
