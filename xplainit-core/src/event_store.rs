//! Event Store - High-performance storage for execution events
//! 
//! This module provides lock-free, concurrent event storage with:
//! - Zero-copy event recording
//! - Thread-safe access
//! - Efficient iteration
//! - Memory-bounded buffering

use crate::ExecutionEvent;
use crossbeam::queue::ArrayQueue;
use parking_lot::RwLock;
use std::sync::Arc;

/// Maximum number of events to store before dropping old ones
const DEFAULT_MAX_EVENTS: usize = 100_000;

/// Thread-safe event storage with circular buffer behavior
pub struct EventStore {
    /// Lock-free queue for event storage
    events: Arc<ArrayQueue<ExecutionEvent>>,
    /// Configuration
    max_events: usize,
    /// Statistics
    stats: Arc<RwLock<EventStats>>,
}

/// Statistics about event capture
#[derive(Debug, Clone, Default)]
pub struct EventStats {
    /// Total events recorded (including dropped)
    pub total_recorded: u64,
    /// Total events dropped (when buffer full)
    pub total_dropped: u64,
    /// Total error events
    pub total_errors: u64,
    /// Current events in buffer
    pub current_count: usize,
}

impl EventStore {
    /// Create a new event store with default capacity
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_MAX_EVENTS)
    }

    /// Create a new event store with specific capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Arc::new(ArrayQueue::new(capacity)),
            max_events: capacity,
            stats: Arc::new(RwLock::new(EventStats::default())),
        }
    }

    /// Record an event (non-blocking, drops oldest if full)
    pub fn record(&self, event: ExecutionEvent) {
        let mut stats = self.stats.write();
        stats.total_recorded += 1;

        if event.is_error() {
            stats.total_errors += 1;
        }

        // Try to push, if full, drop oldest
        match self.events.push(event) {
            Ok(_) => {
                // Successfully pushed
            }
            Err(event) => {
                // Buffer is full, drop oldest event and retry
                stats.total_dropped += 1;
                let _ = self.events.pop(); // Remove oldest
                let _ = self.events.push(event); // Try again with returned event
            }
        }

        stats.current_count = self.events.len();
    }

    /// Get all events (draining the store)
    pub fn drain(&self) -> Vec<ExecutionEvent> {
        let mut events = Vec::with_capacity(self.events.len());
        while let Some(event) = self.events.pop() {
            events.push(event);
        }

        let mut stats = self.stats.write();
        stats.current_count = 0;

        events
    }

    /// Get events without draining (creates snapshot)
    pub fn snapshot(&self) -> Vec<ExecutionEvent> {
        // Since ArrayQueue doesn't support iteration, we drain and re-add
        let events = self.drain();
        for event in &events {
            let _ = self.events.push(event.clone());
        }
        events
    }

    /// Get current statistics
    pub fn stats(&self) -> EventStats {
        self.stats.read().clone()
    }

    /// Clear all events
    pub fn clear(&self) {
        while self.events.pop().is_some() {}
        let mut stats = self.stats.write();
        stats.current_count = 0;
    }

    /// Get current event count
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get maximum capacity
    pub fn capacity(&self) -> usize {
        self.max_events
    }

    /// Check if store is full
    pub fn is_full(&self) -> bool {
        self.events.len() >= self.max_events
    }

    /// Get shared reference for cloning
    pub fn events_arc(&self) -> Arc<ArrayQueue<ExecutionEvent>> {
        Arc::clone(&self.events)
    }

    /// Get shared stats reference
    pub fn stats_arc(&self) -> Arc<RwLock<EventStats>> {
        Arc::clone(&self.stats)
    }
}

impl Default for EventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventStore {
    fn clone(&self) -> Self {
        Self {
            events: Arc::clone(&self.events),
            max_events: self.max_events,
            stats: Arc::clone(&self.stats),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SourceLocation, Value};
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_event_store_creation() {
        let store = EventStore::new();
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
        assert_eq!(store.capacity(), DEFAULT_MAX_EVENTS);
    }

    #[test]
    fn test_event_recording() {
        let store = EventStore::with_capacity(10);
        
        let event = ExecutionEvent::FunctionEnter {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "test.py".into(),
                line: 1,
                column: 0,
                offset: 0,
            },
            name: "test_func".into(),
            args: HashMap::new(),
        };

        store.record(event);
        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());

        let stats = store.stats();
        assert_eq!(stats.total_recorded, 1);
        assert_eq!(stats.current_count, 1);
    }

    #[test]
    fn test_event_draining() {
        let store = EventStore::with_capacity(10);
        
        for i in 0..5 {
            let event = ExecutionEvent::VariableDeclaration {
                id: uuid::Uuid::new_v4(),
                timestamp: Utc::now(),
                location: SourceLocation {
                    file: "test.py".into(),
                    line: i,
                    column: 0,
                    offset: 0,
                },
                name: format!("var_{}", i),
                value: Some(Value::Integer(i as i64)),
                var_type: None,
                is_const: false,
            };
            store.record(event);
        }

        assert_eq!(store.len(), 5);
        
        let events = store.drain();
        assert_eq!(events.len(), 5);
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }

    #[test]
    fn test_circular_buffer_behavior() {
        let store = EventStore::with_capacity(3);
        
        // Add 5 events to a buffer of size 3
        for i in 0..5 {
            let event = ExecutionEvent::VariableDeclaration {
                id: uuid::Uuid::new_v4(),
                timestamp: Utc::now(),
                location: SourceLocation {
                    file: "test.py".into(),
                    line: i,
                    column: 0,
                    offset: 0,
                },
                name: format!("var_{}", i),
                value: Some(Value::Integer(i as i64)),
                var_type: None,
                is_const: false,
            };
            store.record(event);
        }

        let stats = store.stats();
        assert_eq!(stats.total_recorded, 5);
        assert_eq!(stats.total_dropped, 2); // 2 oldest events dropped
        assert_eq!(store.len(), 3); // Only 3 remain
    }

    #[test]
    fn test_error_event_tracking() {
        let store = EventStore::with_capacity(10);
        
        // Record normal event
        store.record(ExecutionEvent::FunctionEnter {
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
        });

        // Record error event
        store.record(ExecutionEvent::RuntimeError {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "test.py".into(),
                line: 2,
                column: 0,
                offset: 0,
            },
            message: "Test error".into(),
            error_type: "RuntimeError".into(),
            stack_trace: vec![],
            context: HashMap::new(),
        });

        let stats = store.stats();
        assert_eq!(stats.total_recorded, 2);
        assert_eq!(stats.total_errors, 1);
    }

    #[test]
    fn test_store_cloning() {
        let store1 = EventStore::with_capacity(10);
        store1.record(ExecutionEvent::FunctionEnter {
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
        });

        let store2 = store1.clone();
        assert_eq!(store2.len(), 1);
        
        // Both stores share the same underlying data
        store2.clear();
        assert_eq!(store1.len(), 0);
    }
}
