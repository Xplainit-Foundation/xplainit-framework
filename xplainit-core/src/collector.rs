//! Event Collector Trait - Interface for language-specific event collection
//! 
//! This module defines the trait that all language-specific collectors must implement.
//! Each language (Python, JavaScript, C, etc.) will have its own collector implementation.

use crate::{ExecutionEvent, Result};
use std::path::PathBuf;

/// Trait for collecting execution events from a running program
/// 
/// Each language implementation will provide its own concrete implementation:
/// - PythonCollector (using sys.settrace)
/// - JavaScriptCollector (using V8 Inspector Protocol)
/// - CCollector (using GDB/LLDB)
/// - etc.
pub trait EventCollector: Send + Sync {
    /// Start collecting events from the target
    /// 
    /// # Arguments
    /// * `target` - The target to instrument (file path, process ID, etc.)
    /// 
    /// # Returns
    /// Result indicating success or failure of instrumentation
    fn start(&mut self, target: &CollectionTarget) -> Result<()>;

    /// Stop collecting events
    fn stop(&mut self) -> Result<()>;

    /// Check if collector is currently active
    fn is_active(&self) -> bool;

    /// Get the next batch of collected events
    /// 
    /// This is called periodically by the runtime engine to retrieve
    /// events captured by the language-specific collector.
    fn collect_events(&mut self) -> Result<Vec<ExecutionEvent>>;

    /// Reset the collector state
    fn reset(&mut self) -> Result<()>;

    /// Get collector statistics
    fn stats(&self) -> CollectorStats;

    /// Configure collector behavior
    fn configure(&mut self, config: CollectorConfig) -> Result<()>;
}

/// Target for event collection
#[derive(Debug, Clone)]
pub enum CollectionTarget {
    /// Collect from a source file
    File(PathBuf),
    
    /// Collect from a running process
    Process {
        pid: u32,
    },
    
    /// Collect from a script/code string
    Code {
        source: String,
        language: String,
    },
    
    /// Collect from a module/library
    Module {
        name: String,
    },
}

/// Configuration for event collectors
#[derive(Debug, Clone)]
pub struct CollectorConfig {
    /// Maximum events to buffer before forcing a flush
    pub max_buffer_size: usize,
    
    /// Whether to collect error events
    pub collect_errors: bool,
    
    /// Whether to collect normal execution events
    pub collect_normal: bool,
    
    /// Maximum call stack depth to trace
    pub max_depth: usize,
    
    /// Whether to trace into standard library
    pub trace_stdlib: bool,
    
    /// Custom filter function names to include
    pub include_functions: Vec<String>,
    
    /// Custom filter function names to exclude
    pub exclude_functions: Vec<String>,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 10_000,
            collect_errors: true,
            collect_normal: true,
            max_depth: 100,
            trace_stdlib: false,
            include_functions: Vec::new(),
            exclude_functions: Vec::new(),
        }
    }
}

/// Statistics about event collection
#[derive(Debug, Clone, Default)]
pub struct CollectorStats {
    /// Total events collected
    pub events_collected: u64,
    
    /// Events currently buffered
    pub events_buffered: usize,
    
    /// Total errors encountered during collection
    pub collection_errors: u64,
    
    /// Whether collector is currently active
    pub is_active: bool,
    
    /// Number of times collection was paused/resumed
    pub pause_count: u64,
}

/// Base struct that can be used by concrete collectors
#[derive(Debug, Clone)]
pub struct BaseCollector {
    pub config: CollectorConfig,
    pub stats: CollectorStats,
    pub is_active: bool,
}

impl BaseCollector {
    pub fn new(config: CollectorConfig) -> Self {
        Self {
            config,
            stats: CollectorStats::default(),
            is_active: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_config_default() {
        let config = CollectorConfig::default();
        assert_eq!(config.max_buffer_size, 10_000);
        assert!(config.collect_errors);
        assert!(config.collect_normal);
        assert_eq!(config.max_depth, 100);
        assert!(!config.trace_stdlib);
    }

    #[test]
    fn test_collection_target_file() {
        let target = CollectionTarget::File(PathBuf::from("test.py"));
        match target {
            CollectionTarget::File(path) => {
                assert_eq!(path.to_str().unwrap(), "test.py");
            }
            _ => panic!("Expected File target"),
        }
    }

    #[test]
    fn test_base_collector_creation() {
        let config = CollectorConfig::default();
        let collector = BaseCollector::new(config);
        assert!(!collector.is_active);
        assert_eq!(collector.stats.events_collected, 0);
    }
}
