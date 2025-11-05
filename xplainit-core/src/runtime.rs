//! Runtime Engine - Core orchestrator for event collection and processing
//! 
//! The RuntimeEngine is the central component that:
//! - Manages event collectors
//! - Processes events through the pipeline
//! - Coordinates with the explanation generator
//! - Handles configuration and lifecycle

use crate::{
    Config, EventStore, ExecutionEvent, Result, XplainitError,
    collector::{EventCollector, CollectionTarget, CollectorStats},
    pipeline::EventPipeline,
};
use parking_lot::RwLock;
use std::sync::Arc;

/// The central runtime engine that orchestrates execution tracing
pub struct RuntimeEngine {
    /// Configuration
    config: Arc<RwLock<Config>>,
    
    /// Event storage
    event_store: EventStore,
    
    /// Active collector (set when tracing is active)
    active_collector: Arc<RwLock<Option<Box<dyn EventCollector>>>>,
    
    /// Event pipeline (filter -> processor -> sink)
    pipeline: Arc<RwLock<Option<EventPipeline>>>,
    
    /// Engine state
    state: Arc<RwLock<EngineState>>,
}

/// State of the runtime engine
#[derive(Debug, Clone, PartialEq)]
pub enum EngineState {
    /// Engine is idle, not collecting events
    Idle,
    
    /// Engine is actively collecting events
    Collecting,
    
    /// Engine is paused (can be resumed)
    Paused,
    
    /// Engine has encountered an error
    Error { message: String },
}

impl RuntimeEngine {
    /// Create a new runtime engine
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            event_store: EventStore::new(),
            active_collector: Arc::new(RwLock::new(None)),
            pipeline: Arc::new(RwLock::new(None)),
            state: Arc::new(RwLock::new(EngineState::Idle)),
        }
    }
    
    /// Set the event pipeline
    pub fn set_pipeline(&self, pipeline: EventPipeline) {
        let mut p = self.pipeline.write();
        *p = Some(pipeline);
    }

    /// Start collecting events using the provided collector
    pub fn start_collection(
        &self,
        mut collector: Box<dyn EventCollector>,
        target: &CollectionTarget,
    ) -> Result<()> {
        // Check if already collecting
        {
            let state = self.state.read();
            if *state == EngineState::Collecting {
                return Err(XplainitError::InternalError(
                    "Engine is already collecting events".into()
                ));
            }
        }

        // Start the collector
        collector.start(target)?;

        // Store the collector
        {
            let mut active = self.active_collector.write();
            *active = Some(collector);
        }

        // Update state
        {
            let mut state = self.state.write();
            *state = EngineState::Collecting;
        }

        Ok(())
    }

    /// Stop collecting events
    pub fn stop_collection(&self) -> Result<()> {
        // Stop the collector
        {
            let mut active = self.active_collector.write();
            if let Some(ref mut collector) = *active {
                collector.stop()?;
            }
            *active = None;
        }

        // Update state
        {
            let mut state = self.state.write();
            *state = EngineState::Idle;
        }

        Ok(())
    }

    /// Pause event collection (can be resumed)
    pub fn pause(&self) -> Result<()> {
        let mut state = self.state.write();
        if *state == EngineState::Collecting {
            *state = EngineState::Paused;
            Ok(())
        } else {
            Err(XplainitError::InternalError(
                "Cannot pause: engine is not collecting".into()
            ))
        }
    }

    /// Resume event collection
    pub fn resume(&self) -> Result<()> {
        let mut state = self.state.write();
        if *state == EngineState::Paused {
            *state = EngineState::Collecting;
            Ok(())
        } else {
            Err(XplainitError::InternalError(
                "Cannot resume: engine is not paused".into()
            ))
        }
    }

    /// Process collected events (called periodically)
    pub fn process_events(&self) -> Result<Vec<ExecutionEvent>> {
        // Only process if actively collecting
        {
            let state = self.state.read();
            if *state != EngineState::Collecting {
                return Ok(Vec::new());
            }
        }

        // Collect events from the active collector
        let events = {
            let mut active = self.active_collector.write();
            if let Some(ref mut collector) = *active {
                collector.collect_events()?
            } else {
                Vec::new()
            }
        };

        // Store events and route through pipeline
        let config = self.config.read().clone();
        for event in &events {
            self.event_store.record(event.clone());
            
            // Route through pipeline if configured
            if let Some(ref mut pipeline) = *self.pipeline.write() {
                let _ = pipeline.handle_event(event.clone(), &config);
            }
        }

        Ok(events)
    }

    /// Get all collected events (draining the store)
    pub fn get_events(&self) -> Vec<ExecutionEvent> {
        self.event_store.drain()
    }

    /// Get a snapshot of events (without draining)
    pub fn snapshot_events(&self) -> Vec<ExecutionEvent> {
        self.event_store.snapshot()
    }

    /// Clear all stored events
    pub fn clear_events(&self) {
        self.event_store.clear();
    }

    /// Get current engine state
    pub fn state(&self) -> EngineState {
        self.state.read().clone()
    }

    /// Check if engine is collecting
    pub fn is_collecting(&self) -> bool {
        *self.state.read() == EngineState::Collecting
    }

    /// Get event store statistics
    pub fn event_stats(&self) -> crate::event_store::EventStats {
        self.event_store.stats()
    }

    /// Get collector statistics (if active)
    pub fn collector_stats(&self) -> Option<CollectorStats> {
        let active = self.active_collector.read();
        active.as_ref().map(|c| c.stats())
    }

    /// Update configuration
    pub fn update_config(&self, config: Config) {
        let mut cfg = self.config.write();
        *cfg = config;
    }

    /// Get current configuration
    pub fn config(&self) -> Config {
        self.config.read().clone()
    }

    /// Get shared config reference
    pub fn config_arc(&self) -> Arc<RwLock<Config>> {
        Arc::clone(&self.config)
    }

    /// Get shared event store reference
    pub fn event_store(&self) -> EventStore {
        self.event_store.clone()
    }
}

impl Clone for RuntimeEngine {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            event_store: self.event_store.clone(),
            active_collector: Arc::clone(&self.active_collector),
            pipeline: Arc::clone(&self.pipeline),
            state: Arc::clone(&self.state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Language;

    #[test]
    fn test_engine_creation() {
        let config = Config::new(Language::Python);
        let engine = RuntimeEngine::new(config);
        assert_eq!(engine.state(), EngineState::Idle);
        assert!(!engine.is_collecting());
    }

    #[test]
    fn test_engine_state_transitions() {
        let config = Config::new(Language::Python);
        let engine = RuntimeEngine::new(config);

        // Initial state is Idle
        assert_eq!(engine.state(), EngineState::Idle);

        // Cannot pause when idle
        assert!(engine.pause().is_err());
    }

    #[test]
    fn test_event_storage() {
        let config = Config::new(Language::Python);
        let engine = RuntimeEngine::new(config);

        // Initially no events
        assert_eq!(engine.get_events().len(), 0);

        // Clear should not panic
        engine.clear_events();

        let stats = engine.event_stats();
        assert_eq!(stats.current_count, 0);
    }

    #[test]
    fn test_engine_cloning() {
        let config = Config::new(Language::Python);
        let engine1 = RuntimeEngine::new(config);
        let engine2 = engine1.clone();

        // Both engines share state
        assert_eq!(engine1.state(), engine2.state());
    }
}
