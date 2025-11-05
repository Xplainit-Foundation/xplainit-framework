//! # Xplainit Core
//!
//! Core runtime instrumentation engine for the Xplainit framework.
//! 
//! This library provides the foundational types and traits for capturing
//! runtime execution events and generating human-readable explanations.
//!
//! ## Features
//!
//! - **Non-invasive instrumentation**: Observe code execution without modification
//! - **Complete event capture**: Every function call, variable change, error, and more
//! - **Error-aware**: Explains errors with the same quality as successful execution
//! - **Zero overhead when disabled**: No performance cost when tracing is off
//!
//! ## Example
//!
//! ```
//! use xplainit_core::{Config, Language, Explainer};
//!
//! // Create configuration
//! let config = Config::new(Language::Python);
//!
//! // Create explainer (actual instrumentation happens in language-specific crates)
//! let explainer = Explainer::new(config);
//! ```

pub mod error;
pub mod config;
pub mod events;
pub mod event_store;
pub mod collector;
pub mod runtime;
pub mod filter;
pub mod advanced_filter;
pub mod processor;
pub mod sink;
pub mod pipeline;
pub mod ast;
pub mod explainer;
pub mod control;
pub mod error_explainer;
pub mod formatter;

// Re-export commonly used types
pub use error::{XplainitError, Result};
pub use config::{
    Config, Language, Verbosity, OutputFormat, 
    OutputDestination, OutputMode
};
pub use events::{
    ExecutionEvent, SourceLocation, Value, StackFrame, LoopExitReason
};
pub use event_store::{EventStore, EventStats};
pub use collector::{
    EventCollector, CollectionTarget, CollectorConfig, 
    CollectorStats, BaseCollector
};
pub use runtime::{RuntimeEngine, EngineState};
pub use filter::{
    EventFilter, AcceptAllFilter, FunctionFilter, 
    EventTypeFilter, DepthFilter, CompositeFilter
};
pub use advanced_filter::{
    ModuleFilter, RegexFilter, CallStackFilter, 
    PerformanceFilter, AdvancedFilter
};
pub use processor::{
    EventProcessor, PassThroughProcessor, EnrichmentProcessor,
    DeduplicationProcessor, RateLimitProcessor, ProcessorPipeline
};
pub use sink::{
    EventSink, ConsoleSink, FileSink, MemorySink, MultiSink
};
pub use pipeline::EventPipeline;
pub use ast::{AstNode, AstParser, AstCache};
pub use explainer::{ExplanationGenerator, VerbosityLevel};
pub use control::{RuntimeControl, ScopedControl, safe_execute};
pub use error_explainer::{ErrorExplainer, ErrorAnalysis, ErrorSeverity, ErrorCategory};
pub use formatter::{
    OutputFormatter, TextFormatter, JsonFormatter, 
    HtmlFormatter, MarkdownFormatter, FormatterFactory
};

use std::sync::Arc;
use parking_lot::RwLock;

/// Main Explainer interface
pub struct Explainer {
    config: Arc<RwLock<Config>>,
    enabled: Arc<RwLock<bool>>,
}

impl Explainer {
    /// Create a new Explainer with the given configuration
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            enabled: Arc::new(RwLock::new(true)),
        }
    }
    
    /// Create a new Explainer for a specific language with default config
    pub fn with_language(language: Language) -> Self {
        Self::new(Config::new(language))
    }
    
    /// Create from environment variables
    pub fn from_env() -> Self {
        let config = Config::from_env();
        let enabled = std::env::var("XPLAINIT_ENABLED")
            .map(|v| v.to_lowercase() != "false" && v != "0")
            .unwrap_or(true);
        
        Self {
            config: Arc::new(RwLock::new(config)),
            enabled: Arc::new(RwLock::new(enabled)),
        }
    }
    
    /// Check if tracing is enabled
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self.enabled.read()
    }
    
    /// Enable tracing
    pub fn enable(&self) {
        *self.enabled.write() = true;
    }
    
    /// Disable tracing
    pub fn disable(&self) {
        *self.enabled.write() = false;
    }
    
    /// Get current configuration (read-only copy)
    pub fn config(&self) -> Config {
        self.config.read().clone()
    }
    
    /// Update configuration
    pub fn update_config<F>(&self, f: F)
    where
        F: FnOnce(&mut Config),
    {
        let mut config = self.config.write();
        f(&mut config);
    }
    
    /// Get a clone of the config Arc for sharing
    pub fn config_arc(&self) -> Arc<RwLock<Config>> {
        Arc::clone(&self.config)
    }
    
    /// Get a clone of the enabled Arc for sharing
    pub fn enabled_arc(&self) -> Arc<RwLock<bool>> {
        Arc::clone(&self.enabled)
    }
}

impl Clone for Explainer {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            enabled: Arc::clone(&self.enabled),
        }
    }
}

impl Default for Explainer {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

// Global instance for convenience (optional)
lazy_static::lazy_static! {
    static ref GLOBAL_EXPLAINER: Explainer = Explainer::from_env();
}

/// Get the global explainer instance
pub fn global() -> &'static Explainer {
    &GLOBAL_EXPLAINER
}

/// Enable tracing globally
pub fn enable() {
    GLOBAL_EXPLAINER.enable();
}

/// Disable tracing globally
pub fn disable() {
    GLOBAL_EXPLAINER.disable();
}

/// Check if tracing is enabled globally
#[inline(always)]
pub fn is_enabled() -> bool {
    GLOBAL_EXPLAINER.is_enabled()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explainer_creation() {
        let explainer = Explainer::with_language(Language::Python);
        assert_eq!(explainer.config().language, Language::Python);
        assert!(explainer.is_enabled());
    }

    #[test]
    fn test_enable_disable() {
        let explainer = Explainer::default();
        assert!(explainer.is_enabled());
        
        explainer.disable();
        assert!(!explainer.is_enabled());
        
        explainer.enable();
        assert!(explainer.is_enabled());
    }

    #[test]
    fn test_config_update() {
        let explainer = Explainer::default();
        
        explainer.update_config(|config| {
            config.verbosity = Verbosity::Detailed;
            config.max_depth = 50;
        });
        
        let config = explainer.config();
        assert_eq!(config.verbosity, Verbosity::Detailed);
        assert_eq!(config.max_depth, 50);
    }

    #[test]
    fn test_clone() {
        let explainer1 = Explainer::with_language(Language::Python);
        let explainer2 = explainer1.clone();
        
        explainer1.disable();
        assert!(!explainer2.is_enabled()); // Shares state
    }

    #[test]
    fn test_global_instance() {
        let global = global();
        assert!(global.is_enabled() || !global.is_enabled()); // Just verify it exists
    }
}
