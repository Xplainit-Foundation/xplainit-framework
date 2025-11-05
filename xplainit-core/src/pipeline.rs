//! Event Pipeline - ties filters, processors, and sinks together

use crate::{ExecutionEvent, Result, Config};
use crate::filter::EventFilter;
use crate::processor::ProcessorPipeline;
use crate::sink::EventSink;

/// Pipeline that accepts events, filters, processes, and sinks them
pub struct EventPipeline {
    filter: Box<dyn EventFilter>,
    processors: ProcessorPipeline,
    sinks: Vec<Box<dyn EventSink>>,
}

impl EventPipeline {
    pub fn new(filter: Box<dyn EventFilter>, processors: ProcessorPipeline) -> Self {
        Self { filter, processors, sinks: Vec::new() }
    }

    pub fn add_sink(mut self, sink: Box<dyn EventSink>) -> Self {
        self.sinks.push(sink);
        self
    }

    /// Handle a single event: filter -> processors -> sinks
    pub fn handle_event(&mut self, event: ExecutionEvent, config: &Config) -> Result<()> {
        // Filter
        if !self.filter.should_capture(&event, config) {
            return Ok(());
        }

        // Process
        match self.processors.process(event)? {
            Some(ev) => {
                // Sink
                for s in &mut self.sinks {
                    let _ = s.write(&ev);
                }
            }
            None => {
                // Event dropped by processor
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Config};
    use crate::filter::AcceptAllFilter;
    use crate::processor::PassThroughProcessor;
    use crate::sink::MemorySink;
    use crate::SourceLocation;
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_pipeline_basic() {
        let filter = Box::new(AcceptAllFilter);
        let processors = ProcessorPipeline::new().add_processor(Box::new(PassThroughProcessor));
        let mem = MemorySink::new(100);
        // Clone the memory sink so we can both insert into pipeline and inspect the original
        let mem_clone = mem.clone();
        let mem_box: Box<dyn crate::sink::EventSink> = Box::new(mem_clone);
        
        let mut pipeline = EventPipeline::new(filter, processors).add_sink(mem_box);
        let config = Config::new(crate::Language::Python);

        let event = crate::ExecutionEvent::FunctionEnter {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation { file: "t".into(), line: 1, column: 0, offset: 0 },
            name: "f".into(),
            args: HashMap::new(),
        };

        pipeline.handle_event(event.clone(), &config).unwrap();

        // Original mem contains the shared Arc and should have received the event
        let events = mem.get_events();
        assert_eq!(events.len(), 1);
    }
}
