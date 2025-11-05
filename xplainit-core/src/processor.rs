//! Event Processor - Transform and enrich events
//! 
//! Processors take raw events and enrich them with additional context,
//! transform them, or perform side effects.

use crate::{ExecutionEvent, Result};

/// Trait for processing events
pub trait EventProcessor: Send + Sync {
    /// Process an event, potentially transforming it
    /// Returns None if the event should be dropped
    fn process(&mut self, event: ExecutionEvent) -> Result<Option<ExecutionEvent>>;
    
    /// Reset processor state
    fn reset(&mut self);
    
    /// Get processor description
    fn description(&self) -> String;
}

/// Pass-through processor (no modification)
#[derive(Debug, Clone, Default)]
pub struct PassThroughProcessor;

impl EventProcessor for PassThroughProcessor {
    fn process(&mut self, event: ExecutionEvent) -> Result<Option<ExecutionEvent>> {
        Ok(Some(event))
    }
    
    fn reset(&mut self) {
        // Nothing to reset
    }
    
    fn description(&self) -> String {
        "Pass-through processor".to_string()
    }
}

/// Enrichment processor - adds context to events
#[derive(Debug, Clone)]
pub struct EnrichmentProcessor {
    /// Track function call depth
    call_depth: usize,
    /// Track execution time
    start_time: Option<std::time::Instant>,
}

impl Default for EnrichmentProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl EnrichmentProcessor {
    pub fn new() -> Self {
        Self {
            call_depth: 0,
            start_time: None,
        }
    }
}

impl EventProcessor for EnrichmentProcessor {
    fn process(&mut self, event: ExecutionEvent) -> Result<Option<ExecutionEvent>> {
        // Track depth
        match &event {
            ExecutionEvent::FunctionEnter { .. } => {
                self.call_depth += 1;
            }
            ExecutionEvent::FunctionExit { .. } => {
                if self.call_depth > 0 {
                    self.call_depth -= 1;
                }
            }
            _ => {}
        }
        
        // Start timer on first event
        if self.start_time.is_none() {
            self.start_time = Some(std::time::Instant::now());
        }
        
        // TODO: Add enrichment data to event
        // For now, just pass through
        Ok(Some(event))
    }
    
    fn reset(&mut self) {
        self.call_depth = 0;
        self.start_time = None;
    }
    
    fn description(&self) -> String {
        format!("Enrichment processor (depth: {})", self.call_depth)
    }
}

/// Deduplication processor - removes duplicate events
#[derive(Debug, Clone)]
pub struct DeduplicationProcessor {
    /// Track seen event IDs
    seen_ids: std::collections::HashSet<uuid::Uuid>,
    max_cache_size: usize,
}

impl DeduplicationProcessor {
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            seen_ids: std::collections::HashSet::new(),
            max_cache_size,
        }
    }
}

impl Default for DeduplicationProcessor {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl EventProcessor for DeduplicationProcessor {
    fn process(&mut self, event: ExecutionEvent) -> Result<Option<ExecutionEvent>> {
        let event_id = event.id();
        
        // Check if we've seen this event
        if self.seen_ids.contains(event_id) {
            return Ok(None); // Drop duplicate
        }
        
        // Add to seen set
        self.seen_ids.insert(*event_id);
        
        // Limit cache size
        if self.seen_ids.len() > self.max_cache_size {
            // Clear oldest entries (simple approach: clear all)
            self.seen_ids.clear();
            self.seen_ids.insert(*event_id);
        }
        
        Ok(Some(event))
    }
    
    fn reset(&mut self) {
        self.seen_ids.clear();
    }
    
    fn description(&self) -> String {
        format!("Deduplication processor ({} cached)", self.seen_ids.len())
    }
}

/// Rate limiting processor - limits event throughput
#[derive(Debug, Clone)]
pub struct RateLimitProcessor {
    max_events_per_second: usize,
    event_count: usize,
    window_start: Option<std::time::Instant>,
}

impl RateLimitProcessor {
    pub fn new(max_events_per_second: usize) -> Self {
        Self {
            max_events_per_second,
            event_count: 0,
            window_start: None,
        }
    }
}

impl Default for RateLimitProcessor {
    fn default() -> Self {
        Self::new(10_000) // 10k events per second default
    }
}

impl EventProcessor for RateLimitProcessor {
    fn process(&mut self, event: ExecutionEvent) -> Result<Option<ExecutionEvent>> {
        let now = std::time::Instant::now();
        
        // Initialize window
        if self.window_start.is_none() {
            self.window_start = Some(now);
            self.event_count = 0;
        }
        
        // Check if we need to reset the window
        if let Some(start) = self.window_start {
            if now.duration_since(start).as_secs() >= 1 {
                // Reset window
                self.window_start = Some(now);
                self.event_count = 0;
            }
        }
        
        // Check rate limit
        if self.event_count >= self.max_events_per_second {
            return Ok(None); // Drop event due to rate limit
        }
        
        self.event_count += 1;
        Ok(Some(event))
    }
    
    fn reset(&mut self) {
        self.event_count = 0;
        self.window_start = None;
    }
    
    fn description(&self) -> String {
        format!(
            "Rate limit processor ({}/{} events/sec)",
            self.event_count, self.max_events_per_second
        )
    }
}

/// Pipeline of multiple processors
#[derive(Default)]
pub struct ProcessorPipeline {
    processors: Vec<Box<dyn EventProcessor>>,
}

impl ProcessorPipeline {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }
    
    pub fn add_processor(mut self, processor: Box<dyn EventProcessor>) -> Self {
        self.processors.push(processor);
        self
    }
    
    pub fn process(&mut self, mut event: ExecutionEvent) -> Result<Option<ExecutionEvent>> {
        for processor in &mut self.processors {
            match processor.process(event)? {
                Some(e) => event = e,
                None => return Ok(None), // Event was filtered out
            }
        }
        Ok(Some(event))
    }
    
    pub fn reset(&mut self) {
        for processor in &mut self.processors {
            processor.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SourceLocation};
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_pass_through_processor() {
        let mut processor = PassThroughProcessor;
        
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
        
        let result = processor.process(event.clone()).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_enrichment_processor() {
        let mut processor = EnrichmentProcessor::new();
        
        let enter = ExecutionEvent::FunctionEnter {
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
        
        processor.process(enter).unwrap();
        assert_eq!(processor.call_depth, 1);
        
        let exit = ExecutionEvent::FunctionExit {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            name: "test".into(),
            return_value: None,
            duration: std::time::Duration::from_millis(10),
        };
        
        processor.process(exit).unwrap();
        assert_eq!(processor.call_depth, 0);
    }

    #[test]
    fn test_deduplication_processor() {
        let mut processor = DeduplicationProcessor::new(100);
        
        let event_id = uuid::Uuid::new_v4();
        
        let event1 = ExecutionEvent::FunctionEnter {
            id: event_id,
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
        
        let event2 = event1.clone();
        
        // First event should pass
        let result1 = processor.process(event1).unwrap();
        assert!(result1.is_some());
        
        // Duplicate should be filtered
        let result2 = processor.process(event2).unwrap();
        assert!(result2.is_none());
    }

    #[test]
    fn test_rate_limit_processor() {
        let mut processor = RateLimitProcessor::new(2); // Only 2 events per second
        
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
        
        // First two events should pass
        assert!(processor.process(event.clone()).unwrap().is_some());
        assert!(processor.process(event.clone()).unwrap().is_some());
        
        // Third event should be rate limited
        assert!(processor.process(event.clone()).unwrap().is_none());
    }
}
