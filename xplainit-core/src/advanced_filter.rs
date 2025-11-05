//! Advanced Filtering System
//! Module-level, regex, call stack, and performance filters

use crate::ExecutionEvent;
use regex::Regex;
use std::collections::{HashSet, HashMap};

/// Module-level filter for including/excluding entire modules
#[derive(Debug, Clone)]
pub struct ModuleFilter {
    /// Module paths to include (e.g., "myapp.core", "myapp.utils")
    /// If empty, all modules are included by default
    pub include_modules: HashSet<String>,
    /// Module paths to exclude (e.g., "site-packages", "lib/python3")
    pub exclude_modules: HashSet<String>,
    /// Patterns for standard library exclusion
    pub exclude_stdlib: bool,
    /// Custom patterns to exclude (glob-style)
    pub exclude_patterns: Vec<String>,
}

impl ModuleFilter {
    pub fn new() -> Self {
        Self {
            include_modules: HashSet::new(),
            exclude_modules: HashSet::new(),
            exclude_stdlib: true,
            exclude_patterns: vec![],
        }
    }
    
    /// Add a module to include list
    pub fn include_module(mut self, module: &str) -> Self {
        self.include_modules.insert(module.to_string());
        self
    }
    
    /// Add a module to exclude list
    pub fn exclude_module(mut self, module: &str) -> Self {
        self.exclude_modules.insert(module.to_string());
        self
    }
    
    /// Add an exclude pattern (e.g., "*/tests/*", "*/vendor/*")
    pub fn exclude_pattern(mut self, pattern: &str) -> Self {
        self.exclude_patterns.push(pattern.to_string());
        self
    }
    
    /// Check if a file path should be filtered out
    pub fn should_filter_file(&self, file_path: &str) -> bool {
        // Check exact module exclusions
        for excluded in &self.exclude_modules {
            if file_path.contains(excluded) {
                return true;
            }
        }
        
        // Check stdlib exclusion
        if self.exclude_stdlib {
            let stdlib_patterns = [
                "site-packages",
                "/lib/python",
                "/lib64/python",
                "\\Lib\\",
                "node_modules",
                "/usr/lib",
                "/usr/local/lib",
                "C:\\Windows\\System32",
            ];
            
            for pattern in &stdlib_patterns {
                if file_path.contains(pattern) {
                    return true;
                }
            }
        }
        
        // Check custom patterns (simple glob matching)
        for pattern in &self.exclude_patterns {
            if pattern.contains('*') {
                // Simple glob matching
                let regex_pattern = pattern
                    .replace("**", ".+")
                    .replace('*', "[^/\\\\]+")
                    .replace('?', ".");
                if let Ok(re) = Regex::new(&regex_pattern) {
                    if re.is_match(file_path) {
                        return true;
                    }
                }
            } else if file_path.contains(pattern) {
                return true;
            }
        }
        
        // Check include list (if specified)
        if !self.include_modules.is_empty() {
            let mut included = false;
            for module in &self.include_modules {
                if file_path.contains(module) {
                    included = true;
                    break;
                }
            }
            return !included;
        }
        
        false
    }
}

impl Default for ModuleFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Regex-based filter for function names
#[derive(Debug)]
pub struct RegexFilter {
    /// Regex patterns to include
    include_patterns: Vec<Regex>,
    /// Regex patterns to exclude
    exclude_patterns: Vec<Regex>,
}

impl RegexFilter {
    pub fn new() -> Self {
        Self {
            include_patterns: vec![],
            exclude_patterns: vec![],
        }
    }
    
    /// Add an include pattern (e.g., r"^test_.*" to include all test functions)
    pub fn include_pattern(mut self, pattern: &str) -> Result<Self, regex::Error> {
        let regex = Regex::new(pattern)?;
        self.include_patterns.push(regex);
        Ok(self)
    }
    
    /// Add an exclude pattern (e.g., r"^_.*" to exclude private functions)
    pub fn exclude_pattern(mut self, pattern: &str) -> Result<Self, regex::Error> {
        let regex = Regex::new(pattern)?;
        self.exclude_patterns.push(regex);
        Ok(self)
    }
    
    /// Check if a function name should be included
    pub fn should_include_function(&self, function_name: &str) -> bool {
        // Check exclusions first
        for pattern in &self.exclude_patterns {
            if pattern.is_match(function_name) {
                return false;
            }
        }
        
        // Check inclusions (if any specified)
        if !self.include_patterns.is_empty() {
            for pattern in &self.include_patterns {
                if pattern.is_match(function_name) {
                    return true;
                }
            }
            return false; // No include pattern matched
        }
        
        true // No include patterns specified, allow by default
    }
}

impl Default for RegexFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Call stack depth filter
#[derive(Debug, Clone)]
pub struct CallStackFilter {
    /// Maximum call stack depth to trace (0 = unlimited)
    pub max_depth: usize,
    /// Current depth tracking per thread
    depth_tracking: HashMap<String, usize>,
    /// Whether to count depth from root or from first instrumented call
    pub count_from_root: bool,
}

impl CallStackFilter {
    pub fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            depth_tracking: HashMap::new(),
            count_from_root: false,
        }
    }
    
    /// Check if event should be captured based on depth
    pub fn should_capture_at_depth(&mut self, thread_id: &str, event: &ExecutionEvent) -> bool {
        if self.max_depth == 0 {
            return true; // No depth limit
        }
        
        // Track depth changes
        match event {
            ExecutionEvent::FunctionEnter { .. } => {
                let depth = self.depth_tracking.entry(thread_id.to_string()).or_insert(0);
                *depth += 1;
                *depth <= self.max_depth
            }
            ExecutionEvent::FunctionExit { .. } => {
                let depth = self.depth_tracking.entry(thread_id.to_string()).or_insert(1);
                if *depth > 0 {
                    *depth -= 1;
                }
                true // Always capture exit events to maintain balance
            }
            _ => {
                // For other events, check current depth
                self.depth_tracking
                    .get(thread_id)
                    .map(|d| *d <= self.max_depth)
                    .unwrap_or(true)
            }
        }
    }
    
    /// Reset depth tracking for a thread
    pub fn reset_thread(&mut self, thread_id: &str) {
        self.depth_tracking.remove(thread_id);
    }
    
    /// Get current depth for a thread
    pub fn current_depth(&self, thread_id: &str) -> usize {
        self.depth_tracking.get(thread_id).copied().unwrap_or(0)
    }
}

impl Default for CallStackFilter {
    fn default() -> Self {
        Self::new(0) // No limit by default
    }
}

/// Performance-critical path filter
/// Excludes hot loops and performance-sensitive code
#[derive(Debug, Clone)]
pub struct PerformanceFilter {
    /// Functions marked as hot/performance-critical
    hot_functions: HashSet<String>,
    /// File patterns for performance-critical code
    hot_paths: Vec<String>,
    /// Whether to exclude loop bodies
    exclude_loops: bool,
    /// Sampling rate (1 = capture all, 10 = capture every 10th event)
    sampling_rate: usize,
    /// Event counter for sampling
    event_count: usize,
}

impl PerformanceFilter {
    pub fn new() -> Self {
        Self {
            hot_functions: HashSet::new(),
            hot_paths: vec![],
            exclude_loops: false,
            sampling_rate: 1,
            event_count: 0,
        }
    }
    
    /// Mark a function as hot/performance-critical
    pub fn mark_hot_function(mut self, function_name: &str) -> Self {
        self.hot_functions.insert(function_name.to_string());
        self
    }
    
    /// Add a hot path pattern
    pub fn mark_hot_path(mut self, path_pattern: &str) -> Self {
        self.hot_paths.push(path_pattern.to_string());
        self
    }
    
    /// Set sampling rate (capture 1 in N events)
    pub fn with_sampling(mut self, rate: usize) -> Self {
        self.sampling_rate = rate.max(1);
        self
    }
    
    /// Exclude loop iterations
    pub fn exclude_loop_bodies(mut self) -> Self {
        self.exclude_loops = true;
        self
    }
    
    /// Check if event should be sampled
    pub fn should_sample(&mut self) -> bool {
        self.event_count += 1;
        if self.sampling_rate == 1 {
            return true;
        }
        self.event_count.is_multiple_of(self.sampling_rate)
    }
    
    /// Check if event is in a hot path
    pub fn is_hot_event(&self, event: &ExecutionEvent) -> bool {
        // Check if function is marked as hot
        match event {
            ExecutionEvent::FunctionEnter { name, .. } | ExecutionEvent::FunctionExit { name, .. } => {
                if self.hot_functions.contains(name) {
                    return true;
                }
            }
            ExecutionEvent::LoopIteration { .. } if self.exclude_loops => {
                return true;
            }
            _ => {}
        }
        
        // Check file path against hot paths
        let location = event.location();
        for pattern in &self.hot_paths {
            if location.file.contains(pattern) {
                return true;
            }
        }
        
        false
    }
}

impl Default for PerformanceFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Combined advanced filter
#[derive(Debug)]
pub struct AdvancedFilter {
    pub module_filter: ModuleFilter,
    pub regex_filter: RegexFilter,
    pub call_stack_filter: CallStackFilter,
    pub performance_filter: PerformanceFilter,
}

impl AdvancedFilter {
    pub fn new() -> Self {
        Self {
            module_filter: ModuleFilter::new(),
            regex_filter: RegexFilter::new(),
            call_stack_filter: CallStackFilter::new(0),
            performance_filter: PerformanceFilter::new(),
        }
    }
    
    /// Builder: set module filter
    pub fn with_module_filter(mut self, filter: ModuleFilter) -> Self {
        self.module_filter = filter;
        self
    }
    
    /// Builder: set regex filter
    pub fn with_regex_filter(mut self, filter: RegexFilter) -> Self {
        self.regex_filter = filter;
        self
    }
    
    /// Builder: set call stack filter
    pub fn with_call_stack_filter(mut self, filter: CallStackFilter) -> Self {
        self.call_stack_filter = filter;
        self
    }
    
    /// Builder: set performance filter
    pub fn with_performance_filter(mut self, filter: PerformanceFilter) -> Self {
        self.performance_filter = filter;
        self
    }
    
    /// Check if event should be captured (combines all filters)
    pub fn should_capture(&mut self, event: &ExecutionEvent, thread_id: &str) -> bool {
        // Module filter
        let location = event.location();
        if self.module_filter.should_filter_file(&location.file) {
            return false;
        }
        
        // Function name regex filter
        match event {
            ExecutionEvent::FunctionEnter { name, .. } | ExecutionEvent::FunctionExit { name, .. } => {
                if !self.regex_filter.should_include_function(name) {
                    return false;
                }
            }
            _ => {}
        }
        
        // Call stack depth filter
        if !self.call_stack_filter.should_capture_at_depth(thread_id, event) {
            return false;
        }
        
        // Performance filter (hot paths and sampling)
        if self.performance_filter.is_hot_event(event) {
            return false; // Skip hot paths entirely
        }
        
        if !self.performance_filter.should_sample() {
            return false; // Failed sampling check
        }
        
        true
    }
}

impl Default for AdvancedFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceLocation;
    use chrono::Utc;
    use uuid::Uuid;
    use std::collections::HashMap;

    #[test]
    fn test_module_filter_stdlib() {
        let filter = ModuleFilter::new();
        
        assert!(filter.should_filter_file("/usr/lib/python3.9/os.py"));
        assert!(filter.should_filter_file("C:\\Python39\\Lib\\json\\decoder.py"));
        assert!(filter.should_filter_file("/app/venv/lib/site-packages/requests/api.py"));
        assert!(!filter.should_filter_file("/app/myapp/core.py"));
    }
    
    #[test]
    fn test_module_filter_exclude() {
        let filter = ModuleFilter::new()
            .exclude_module("tests")
            .exclude_module("vendor");
        
        assert!(filter.should_filter_file("/app/tests/test_core.py"));
        assert!(filter.should_filter_file("/app/vendor/library.py"));
        assert!(!filter.should_filter_file("/app/src/core.py"));
    }
    
    #[test]
    fn test_module_filter_include() {
        let filter = ModuleFilter {
            include_modules: {
                let mut set = HashSet::new();
                set.insert("myapp".to_string());
                set
            },
            exclude_modules: HashSet::new(),
            exclude_stdlib: false,
            exclude_patterns: vec![],
        };
        
        assert!(!filter.should_filter_file("/app/myapp/core.py"));
        assert!(filter.should_filter_file("/app/other/utils.py"));
    }
    
    #[test]
    fn test_regex_filter_include() {
        let filter = RegexFilter::new()
            .include_pattern(r"^test_.*").unwrap();
        
        assert!(filter.should_include_function("test_example"));
        assert!(filter.should_include_function("test_another"));
        assert!(!filter.should_include_function("example"));
    }
    
    #[test]
    fn test_regex_filter_exclude() {
        let filter = RegexFilter::new()
            .exclude_pattern(r"^_.*").unwrap();
        
        assert!(!filter.should_include_function("_private"));
        assert!(!filter.should_include_function("__internal__"));
        assert!(filter.should_include_function("public"));
    }
    
    #[test]
    fn test_call_stack_filter() {
        let mut filter = CallStackFilter::new(3);
        let thread_id = "thread-1";
        
        let enter = ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "func".to_string(),
            args: HashMap::new(),
            location: SourceLocation { file: "test.py".to_string(), line: 1, column: 0, offset: 0 },
            timestamp: Utc::now(),
        };
        
        // Depth 1
        assert!(filter.should_capture_at_depth(thread_id, &enter));
        assert_eq!(filter.current_depth(thread_id), 1);
        
        // Depth 2
        assert!(filter.should_capture_at_depth(thread_id, &enter));
        assert_eq!(filter.current_depth(thread_id), 2);
        
        // Depth 3
        assert!(filter.should_capture_at_depth(thread_id, &enter));
        assert_eq!(filter.current_depth(thread_id), 3);
        
        // Depth 4 - should be rejected
        assert!(!filter.should_capture_at_depth(thread_id, &enter));
        assert_eq!(filter.current_depth(thread_id), 4); // Tracked but rejected
    }
    
    #[test]
    fn test_performance_filter_hot_functions() {
        let filter = PerformanceFilter::new()
            .mark_hot_function("tight_loop")
            .mark_hot_function("inner_calc");
        
        let event = ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "tight_loop".to_string(),
            args: HashMap::new(),
            location: SourceLocation { file: "hot.py".to_string(), line: 1, column: 0, offset: 0 },
            timestamp: Utc::now(),
        };
        
        assert!(filter.is_hot_event(&event));
    }
    
    #[test]
    fn test_performance_filter_sampling() {
        let mut filter = PerformanceFilter::new().with_sampling(3);
        
        // First call: count=1, 1 % 3 = 1 (not captured)
        assert!(!filter.should_sample());
        // Second call: count=2, 2 % 3 = 2 (not captured)
        assert!(!filter.should_sample());
        // Third call: count=3, 3 % 3 = 0 (captured!)
        assert!(filter.should_sample());
        // Fourth call: count=4, 4 % 3 = 1 (not captured)
        assert!(!filter.should_sample());
    }
}
