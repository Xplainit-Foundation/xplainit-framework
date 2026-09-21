//! Event Filter - Selective event capture
//!
//! Filters determine which events should be captured and which should be ignored.
//! This allows for precise control over tracing scope and performance.

use crate::events::Value;
use crate::{Config, ExecutionEvent};
use chrono::{DateTime, Utc};
use regex::Regex;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};

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
            | ExecutionEvent::LoopExit { .. } => self.capture_loops,
            _ if event.is_error() => self.capture_errors,
            _ => self.capture_normal,
        }
    }

    fn description(&self) -> String {
        format!(
            "Event type filter (functions: {}, variables: {}, loops: {}, errors: {})",
            self.capture_functions, self.capture_variables, self.capture_loops, self.capture_errors
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
            ExecutionEvent::FunctionEnter { .. } => self.current_depth < self.max_depth,
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

/// Filter that only captures events whose timestamp falls within a window.
///
/// Both bounds are optional so the window can be open-ended:
/// * `start == None` means "no lower bound".
/// * `end == None` means "no upper bound".
///
/// The window is inclusive of both bounds.
#[derive(Debug, Clone, Default)]
pub struct TimeRangeFilter {
    /// Inclusive lower bound (open when `None`).
    pub start: Option<DateTime<Utc>>,
    /// Inclusive upper bound (open when `None`).
    pub end: Option<DateTime<Utc>>,
}

impl TimeRangeFilter {
    /// Create a filter with both bounds open (captures everything).
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
        }
    }

    /// Set the inclusive lower bound.
    pub fn with_start(mut self, start: DateTime<Utc>) -> Self {
        self.start = Some(start);
        self
    }

    /// Set the inclusive upper bound.
    pub fn with_end(mut self, end: DateTime<Utc>) -> Self {
        self.end = Some(end);
        self
    }

    /// Create a filter with both inclusive bounds set.
    pub fn between(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            start: Some(start),
            end: Some(end),
        }
    }
}

impl EventFilter for TimeRangeFilter {
    fn should_capture(&self, event: &ExecutionEvent, _config: &Config) -> bool {
        let ts = event.timestamp();
        if let Some(start) = self.start {
            if *ts < start {
                return false;
            }
        }
        if let Some(end) = self.end {
            if *ts > end {
                return false;
            }
        }
        true
    }

    fn description(&self) -> String {
        format!(
            "Time range filter (start: {}, end: {})",
            self.start
                .map(|t| t.to_rfc3339())
                .unwrap_or_else(|| "open".to_string()),
            self.end
                .map(|t| t.to_rfc3339())
                .unwrap_or_else(|| "open".to_string())
        )
    }
}

/// Render a `Value` to a plain string for substring matching.
///
/// This intentionally mirrors a debug-style rendering rather than any
/// language-specific formatting so the filter is predictable across bindings.
fn render_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(items) => {
            let rendered: Vec<String> = items.iter().map(render_value).collect();
            format!("[{}]", rendered.join(", "))
        }
        Value::Object(map) => {
            let mut entries: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, render_value(v)))
                .collect();
            // Sort so rendering is deterministic regardless of hash ordering.
            entries.sort();
            format!("{{{}}}", entries.join(", "))
        }
        Value::Function(name) => name.clone(),
        Value::Unknown(s) => s.clone(),
    }
}

/// Filter that matches on the value payloads reachable from an event.
///
/// An event can carry values in several variants (variable assignments,
/// declarations, function exits/enters, and type errors). This filter inspects
/// all such values and matches when either:
/// * the rendered value contains `substring` (when set), or
/// * the `Value::type_name()` equals `type_name` (when set).
///
/// If neither criterion is set, any event that carries a value matches.
/// Events that carry no inspectable value use `pass_without_value` (default
/// `true`) so unrelated events are not silently dropped.
#[derive(Debug, Clone)]
pub struct ValueFilter {
    /// Substring to look for in the rendered value.
    pub substring: Option<String>,
    /// Required `Value::type_name()` (e.g. "string", "integer").
    pub type_name: Option<String>,
    /// Whether events without any inspectable value pass through.
    pub pass_without_value: bool,
}

impl ValueFilter {
    /// Create an empty filter (matches any event carrying a value, passes
    /// through events without one).
    pub fn new() -> Self {
        Self {
            substring: None,
            type_name: None,
            pass_without_value: true,
        }
    }

    /// Match values whose rendering contains `substring`.
    pub fn with_substring(mut self, substring: impl Into<String>) -> Self {
        self.substring = Some(substring.into());
        self
    }

    /// Match values whose `type_name()` equals `type_name`.
    pub fn with_type(mut self, type_name: impl Into<String>) -> Self {
        self.type_name = Some(type_name.into());
        self
    }

    /// Control whether events without inspectable values pass through.
    pub fn pass_without_value(mut self, pass: bool) -> Self {
        self.pass_without_value = pass;
        self
    }

    /// Collect every inspectable value reachable from the event.
    fn values_of(event: &ExecutionEvent) -> Vec<Value> {
        match event {
            ExecutionEvent::VariableAssign {
                new_value,
                old_value,
                ..
            } => {
                let mut values = vec![new_value.clone()];
                if let Some(old) = old_value {
                    values.push(old.clone());
                }
                values
            }
            ExecutionEvent::VariableDeclaration { value, .. } => {
                value.clone().into_iter().collect()
            }
            ExecutionEvent::FunctionExit { return_value, .. } => {
                return_value.clone().into_iter().collect()
            }
            ExecutionEvent::Return { value, .. } => value.clone().into_iter().collect(),
            ExecutionEvent::TypeError { value, .. } => vec![value.clone()],
            ExecutionEvent::DivisionByZero { numerator, .. } => vec![numerator.clone()],
            ExecutionEvent::LoopIteration { loop_var_value, .. } => {
                loop_var_value.clone().into_iter().collect()
            }
            ExecutionEvent::FunctionEnter { args, .. } => args.values().cloned().collect(),
            _ => Vec::new(),
        }
    }

    /// Test whether a single value matches the configured criteria.
    fn value_matches(&self, value: &Value) -> bool {
        if let Some(type_name) = &self.type_name {
            if value.type_name() != type_name.as_str() {
                return false;
            }
        }
        if let Some(substring) = &self.substring {
            if !render_value(value).contains(substring.as_str()) {
                return false;
            }
        }
        true
    }
}

impl Default for ValueFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl EventFilter for ValueFilter {
    fn should_capture(&self, event: &ExecutionEvent, _config: &Config) -> bool {
        let values = Self::values_of(event);
        if values.is_empty() {
            return self.pass_without_value;
        }
        values.iter().any(|v| self.value_matches(v))
    }

    fn description(&self) -> String {
        format!(
            "Value filter (substring: {}, type: {}, pass_without_value: {})",
            self.substring.as_deref().unwrap_or("<any>"),
            self.type_name.as_deref().unwrap_or("<any>"),
            self.pass_without_value
        )
    }
}

/// Deterministic 1-in-N sampling filter for high-frequency events.
///
/// Because [`EventFilter::should_capture`] takes `&self`, the counter uses
/// interior mutability via [`AtomicUsize`] so no external randomness (and no
/// `rand` dependency) is required. Sampling is fully deterministic: for a rate
/// of `N`, every `N`-th matching event is captured.
///
/// Events whose type is listed in `always_keep` bypass sampling entirely, which
/// lets callers keep every error while sampling noisy loop iterations.
#[derive(Debug, Default)]
pub struct FrequencyFilter {
    /// Capture 1 in `sampling_rate` events (minimum 1 = capture everything).
    sampling_rate: usize,
    /// Event type names (see [`ExecutionEvent::event_type`]) that always pass.
    always_keep: HashSet<String>,
    /// Whether error events always pass regardless of sampling.
    always_keep_errors: bool,
    /// Monotonic counter for the events subject to sampling.
    counter: AtomicUsize,
}

impl FrequencyFilter {
    /// Create a filter that captures 1 in `rate` events (rate clamped to >= 1).
    pub fn new(rate: usize) -> Self {
        Self {
            sampling_rate: rate.max(1),
            always_keep: HashSet::new(),
            always_keep_errors: false,
            counter: AtomicUsize::new(0),
        }
    }

    /// Always keep events of the given type (by `event_type()` name).
    pub fn always_keep_type(mut self, event_type: impl Into<String>) -> Self {
        self.always_keep.insert(event_type.into());
        self
    }

    /// Always keep error events regardless of sampling.
    pub fn always_keep_errors(mut self, keep: bool) -> Self {
        self.always_keep_errors = keep;
        self
    }
}

impl EventFilter for FrequencyFilter {
    fn should_capture(&self, event: &ExecutionEvent, _config: &Config) -> bool {
        if self.always_keep_errors && event.is_error() {
            return true;
        }
        if self.always_keep.contains(event.event_type()) {
            return true;
        }
        if self.sampling_rate <= 1 {
            return true;
        }
        // fetch_add returns the previous value, so the first sampled event has
        // count 1. Capture when count is a multiple of the rate.
        let count = self.counter.fetch_add(1, Ordering::Relaxed) + 1;
        count.is_multiple_of(self.sampling_rate)
    }

    fn description(&self) -> String {
        format!(
            "Frequency filter (1 in {}, always_keep_types: {}, always_keep_errors: {})",
            self.sampling_rate,
            self.always_keep.len(),
            self.always_keep_errors
        )
    }
}

/// Filter events by their source file path.
///
/// Both include and exclude lists accept plain substrings or simple globs
/// (`**` matches across separators, `*` matches within a path segment, `?`
/// matches a single character), mirroring [`crate::ModuleFilter`]. Exclusions
/// always win over inclusions. When the include list is empty, any path that is
/// not excluded passes.
#[derive(Debug, Clone, Default)]
pub struct PathFilter {
    /// Patterns whose match includes the event.
    pub include: Vec<String>,
    /// Patterns whose match excludes the event (takes precedence).
    pub exclude: Vec<String>,
}

impl PathFilter {
    /// Create an empty filter (includes everything).
    pub fn new() -> Self {
        Self {
            include: Vec::new(),
            exclude: Vec::new(),
        }
    }

    /// Add an include pattern.
    pub fn include(mut self, pattern: impl Into<String>) -> Self {
        self.include.push(pattern.into());
        self
    }

    /// Add an exclude pattern.
    pub fn exclude(mut self, pattern: impl Into<String>) -> Self {
        self.exclude.push(pattern.into());
        self
    }

    /// Test a path against a single pattern (substring or simple glob).
    fn pattern_matches(pattern: &str, path: &str) -> bool {
        if pattern.contains('*') || pattern.contains('?') {
            let regex_pattern = pattern
                .replace("**", ".+")
                .replace('*', "[^/\\\\]+")
                .replace('?', ".");
            if let Ok(re) = Regex::new(&regex_pattern) {
                return re.is_match(path);
            }
            // Fall back to substring matching if the pattern is not valid regex.
            return path.contains(pattern);
        }
        path.contains(pattern)
    }
}

impl EventFilter for PathFilter {
    fn should_capture(&self, event: &ExecutionEvent, _config: &Config) -> bool {
        // Borrow the file path instead of cloning the whole SourceLocation.
        // For variants that carry no location we fall back to the same
        // "<unknown>" sentinel that `location()` used to synthesize, so the
        // filtering decision is byte-for-byte identical to the pre-refactor
        // behavior while avoiding a per-event String clone on the common path.
        const UNKNOWN_FILE: &str = "<unknown>";
        let path: &str = event
            .location_ref()
            .map_or(UNKNOWN_FILE, |loc| loc.file.as_str());

        // Exclusions win over everything.
        if self.exclude.iter().any(|p| Self::pattern_matches(p, path)) {
            return false;
        }

        // With no include list, anything not excluded passes.
        if self.include.is_empty() {
            return true;
        }

        self.include.iter().any(|p| Self::pattern_matches(p, path))
    }

    fn description(&self) -> String {
        format!(
            "Path filter (include: {}, exclude: {})",
            self.include.len(),
            self.exclude.len()
        )
    }
}

/// Filter driven by a user-provided predicate closure.
///
/// This gives callers arbitrary custom filtering logic while still implementing
/// [`EventFilter`], so it composes inside a [`CompositeFilter`] like any other.
/// Boxed predicate signature used by [`UserFilter`].
pub type UserPredicate = Box<dyn Fn(&ExecutionEvent, &Config) -> bool + Send + Sync>;

pub struct UserFilter {
    predicate: UserPredicate,
    label: String,
}

impl UserFilter {
    /// Create a filter from a predicate closure.
    pub fn new<F>(predicate: F) -> Self
    where
        F: Fn(&ExecutionEvent, &Config) -> bool + Send + Sync + 'static,
    {
        Self {
            predicate: Box::new(predicate),
            label: "custom predicate".to_string(),
        }
    }

    /// Create a filter from a predicate closure with a descriptive label.
    pub fn with_label<F>(label: impl Into<String>, predicate: F) -> Self
    where
        F: Fn(&ExecutionEvent, &Config) -> bool + Send + Sync + 'static,
    {
        Self {
            predicate: Box::new(predicate),
            label: label.into(),
        }
    }
}

impl EventFilter for UserFilter {
    fn should_capture(&self, event: &ExecutionEvent, config: &Config) -> bool {
        (self.predicate)(event, config)
    }

    fn description(&self) -> String {
        format!("User filter ({})", self.label)
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
        let filter = FunctionFilter::new().include("allowed_func");

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
        let filter = FunctionFilter::new().exclude("blocked_func");

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

    fn loc() -> SourceLocation {
        SourceLocation {
            file: "test.py".into(),
            line: 1,
            column: 0,
            offset: 0,
        }
    }

    fn assign_at(ts: chrono::DateTime<Utc>, value: Value) -> ExecutionEvent {
        ExecutionEvent::VariableAssign {
            id: uuid::Uuid::new_v4(),
            name: "x".into(),
            old_value: None,
            new_value: value,
            location: loc(),
            timestamp: ts,
        }
    }

    fn loop_iteration() -> ExecutionEvent {
        ExecutionEvent::LoopIteration {
            id: uuid::Uuid::new_v4(),
            loop_type: "for".into(),
            iteration: 0,
            loop_var: None,
            loop_var_value: None,
            timestamp: Utc::now(),
        }
    }

    fn runtime_error() -> ExecutionEvent {
        ExecutionEvent::RuntimeError {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            location: loc(),
            message: "boom".into(),
            error_type: "RuntimeError".into(),
            stack_trace: vec![],
            context: HashMap::new(),
        }
    }

    #[test]
    fn test_time_range_filter_window() {
        let config = Config::new(crate::Language::Python);
        let start = Utc::now();
        let inside = start + chrono::Duration::seconds(5);
        let after = start + chrono::Duration::seconds(20);
        let before = start - chrono::Duration::seconds(5);
        let end = start + chrono::Duration::seconds(10);

        let filter = TimeRangeFilter::between(start, end);

        assert!(filter.should_capture(&assign_at(inside, Value::Integer(1)), &config));
        assert!(filter.should_capture(&assign_at(start, Value::Integer(1)), &config)); // inclusive
        assert!(filter.should_capture(&assign_at(end, Value::Integer(1)), &config)); // inclusive
        assert!(!filter.should_capture(&assign_at(after, Value::Integer(1)), &config));
        assert!(!filter.should_capture(&assign_at(before, Value::Integer(1)), &config));
    }

    #[test]
    fn test_time_range_filter_open_bounds() {
        let config = Config::new(crate::Language::Python);
        let now = Utc::now();

        // Only a lower bound: everything at or after `now` is captured.
        let lower = TimeRangeFilter::new().with_start(now);
        assert!(lower.should_capture(
            &assign_at(now + chrono::Duration::seconds(1), Value::Integer(1)),
            &config
        ));
        assert!(!lower.should_capture(
            &assign_at(now - chrono::Duration::seconds(1), Value::Integer(1)),
            &config
        ));

        // Only an upper bound: everything at or before `now` is captured.
        let upper = TimeRangeFilter::new().with_end(now);
        assert!(upper.should_capture(
            &assign_at(now - chrono::Duration::seconds(1), Value::Integer(1)),
            &config
        ));
        assert!(!upper.should_capture(
            &assign_at(now + chrono::Duration::seconds(1), Value::Integer(1)),
            &config
        ));

        // No bounds: everything passes.
        let open = TimeRangeFilter::new();
        assert!(open.should_capture(&assign_at(now, Value::Integer(1)), &config));
    }

    #[test]
    fn test_value_filter_substring_match() {
        let config = Config::new(crate::Language::Python);
        let filter = ValueFilter::new().with_substring("hello");

        let matching = assign_at(Utc::now(), Value::String("hello world".into()));
        let non_matching = assign_at(Utc::now(), Value::String("goodbye".into()));

        assert!(filter.should_capture(&matching, &config));
        assert!(!filter.should_capture(&non_matching, &config));
    }

    #[test]
    fn test_value_filter_type_match() {
        let config = Config::new(crate::Language::Python);
        let filter = ValueFilter::new().with_type("integer");

        assert!(filter.should_capture(&assign_at(Utc::now(), Value::Integer(42)), &config));
        assert!(!filter.should_capture(&assign_at(Utc::now(), Value::String("42".into())), &config));
    }

    #[test]
    fn test_value_filter_no_value_passthrough() {
        let config = Config::new(crate::Language::Python);
        // A loop iteration with no loop_var_value carries no inspectable value.
        let event = loop_iteration();

        let passing = ValueFilter::new().with_substring("anything");
        assert!(passing.should_capture(&event, &config));

        let dropping = ValueFilter::new()
            .with_substring("anything")
            .pass_without_value(false);
        assert!(!dropping.should_capture(&event, &config));
    }

    #[test]
    fn test_frequency_filter_deterministic_sequence() {
        let config = Config::new(crate::Language::Python);
        let filter = FrequencyFilter::new(3);

        let expected = [false, false, true, false, false, true, false];
        for (i, want) in expected.iter().enumerate() {
            let got = filter.should_capture(&loop_iteration(), &config);
            assert_eq!(got, *want, "unexpected sampling decision at index {}", i);
        }
    }

    #[test]
    fn test_frequency_filter_always_keep_errors() {
        let config = Config::new(crate::Language::Python);
        let filter = FrequencyFilter::new(1000).always_keep_errors(true);

        // Errors always pass and do not consume the sampling counter.
        for _ in 0..5 {
            assert!(filter.should_capture(&runtime_error(), &config));
        }
        // A sampled event still needs 1000 hits before it is captured.
        assert!(!filter.should_capture(&loop_iteration(), &config));
    }

    #[test]
    fn test_frequency_filter_always_keep_type() {
        let config = Config::new(crate::Language::Python);
        let filter = FrequencyFilter::new(2).always_keep_type("loop_iteration");

        // Loop iterations bypass sampling entirely.
        for _ in 0..4 {
            assert!(filter.should_capture(&loop_iteration(), &config));
        }
    }

    #[test]
    fn test_path_filter_include_exclude() {
        let config = Config::new(crate::Language::Python);

        let src = ExecutionEvent::FunctionEnter {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "/app/src/core.py".into(),
                line: 1,
                column: 0,
                offset: 0,
            },
            name: "f".into(),
            args: HashMap::new(),
        };
        let test = ExecutionEvent::FunctionEnter {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "/app/tests/test_core.py".into(),
                line: 1,
                column: 0,
                offset: 0,
            },
            name: "f".into(),
            args: HashMap::new(),
        };

        let filter = PathFilter::new().include("/app/src").exclude("/tests/");

        assert!(filter.should_capture(&src, &config));
        assert!(!filter.should_capture(&test, &config));
    }

    #[test]
    fn test_path_filter_exclusion_wins() {
        let config = Config::new(crate::Language::Python);

        let event = ExecutionEvent::FunctionEnter {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "/app/src/generated.py".into(),
                line: 1,
                column: 0,
                offset: 0,
            },
            name: "f".into(),
            args: HashMap::new(),
        };

        // Even though the include matches, the exclude takes precedence.
        let filter = PathFilter::new().include("/app/src").exclude("generated");
        assert!(!filter.should_capture(&event, &config));
    }

    #[test]
    fn test_path_filter_glob() {
        let config = Config::new(crate::Language::Python);

        let py = ExecutionEvent::FunctionEnter {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "/app/module.py".into(),
                line: 1,
                column: 0,
                offset: 0,
            },
            name: "f".into(),
            args: HashMap::new(),
        };
        let rs = ExecutionEvent::FunctionEnter {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "/app/module.rs".into(),
                line: 1,
                column: 0,
                offset: 0,
            },
            name: "f".into(),
            args: HashMap::new(),
        };

        let filter = PathFilter::new().include("*.py");
        assert!(filter.should_capture(&py, &config));
        assert!(!filter.should_capture(&rs, &config));
    }

    #[test]
    fn test_user_filter_predicate() {
        let config = Config::new(crate::Language::Python);

        // Keep only error events.
        let filter = UserFilter::with_label("errors only", |event, _config| event.is_error());

        assert!(filter.should_capture(&runtime_error(), &config));
        assert!(!filter.should_capture(&loop_iteration(), &config));
        assert_eq!(filter.description(), "User filter (errors only)");
    }

    #[test]
    fn test_new_filters_compose_in_composite() {
        let config = Config::new(crate::Language::Python);

        // require_all: value must be a string AND path must be under /app/src.
        let composite = CompositeFilter::new(true)
            .add_filter(Box::new(ValueFilter::new().with_type("string")))
            .add_filter(Box::new(PathFilter::new().include("test.py")));

        let matching = assign_at(Utc::now(), Value::String("ok".into()));
        let wrong_type = assign_at(Utc::now(), Value::Integer(1));

        assert!(composite.should_capture(&matching, &config));
        assert!(!composite.should_capture(&wrong_type, &config));
    }
}
