//! Event Pipeline - ties filters, processors, and sinks together

use crate::control::RuntimeControl;
use crate::filter::EventFilter;
use crate::processor::ProcessorPipeline;
use crate::sink::EventSink;
use crate::{Config, ExecutionEvent, Result};

/// Pipeline that accepts events, filters, processes, and sinks them
pub struct EventPipeline {
    filter: Box<dyn EventFilter>,
    processors: ProcessorPipeline,
    sinks: Vec<Box<dyn EventSink>>,
    /// Optional runtime control. When present, the pipeline routes its fallible
    /// operations (processor + sink writes) through the circuit-breaker:
    /// framework errors call [`RuntimeControl::record_error`] and a clean event
    /// calls [`RuntimeControl::record_success`]. This is what actually drives
    /// the "auto-disable after N consecutive errors" recovery in a running
    /// program - without a control wired in, the breaker never sees the
    /// pipeline's errors. See `control.rs` module docs.
    control: Option<RuntimeControl>,
}

impl EventPipeline {
    pub fn new(filter: Box<dyn EventFilter>, processors: ProcessorPipeline) -> Self {
        Self {
            filter,
            processors,
            sinks: Vec::new(),
            control: None,
        }
    }

    pub fn add_sink(mut self, sink: Box<dyn EventSink>) -> Self {
        self.sinks.push(sink);
        self
    }

    /// Attach a [`RuntimeControl`] so the pipeline drives the circuit-breaker.
    ///
    /// Once set, every [`Self::handle_event`] call that reaches the
    /// processor/sink stage records either a success (resetting the
    /// consecutive-error counter) or an error (advancing it, tripping the
    /// breaker once the configured threshold is reached). When the breaker is
    /// tripped the control enters panic mode and the runtime stops capturing.
    pub fn with_control(mut self, control: RuntimeControl) -> Self {
        self.control = Some(control);
        self
    }

    /// Handle a single event: filter -> processors -> sinks
    pub fn handle_event(&mut self, event: ExecutionEvent, config: &Config) -> Result<()> {
        // Filter
        if !self.filter.should_capture(&event, config) {
            return Ok(());
        }

        // Process + sink. Any failure is routed through the circuit-breaker so
        // repeated real errors trip it and auto-disable tracing; a clean run
        // records a success so transient failures do not accumulate.
        let outcome = self.process_and_sink(event);
        match &outcome {
            Ok(()) => {
                if let Some(ctrl) = &self.control {
                    ctrl.record_success();
                }
            }
            Err(_) => {
                if let Some(ctrl) = &self.control {
                    ctrl.record_error();
                }
            }
        }
        outcome
    }

    /// Run the processor stage and fan the result out to every sink. Returns
    /// the first error encountered (from a processor or any sink write) so the
    /// caller can record it against the circuit-breaker.
    fn process_and_sink(&mut self, event: ExecutionEvent) -> Result<()> {
        match self.processors.process(event)? {
            Some(ev) => {
                // Sink to every configured destination. A sink failure is a
                // real framework error and is surfaced (rather than swallowed)
                // so the breaker can observe it.
                for s in &mut self.sinks {
                    s.write(&ev)?;
                }
            }
            None => {
                // Event dropped by processor - this is a normal outcome.
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::AcceptAllFilter;
    use crate::processor::PassThroughProcessor;
    use crate::sink::MemorySink;
    use crate::Config;
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
            location: SourceLocation {
                file: "t".into(),
                line: 1,
                column: 0,
                offset: 0,
            },
            name: "f".into(),
            args: HashMap::new(),
        };

        pipeline.handle_event(event.clone(), &config).unwrap();

        // Original mem contains the shared Arc and should have received the event
        let events = mem.get_events();
        assert_eq!(events.len(), 1);
    }

    /// A sink whose `write` always fails, used to drive the pipeline's error
    /// path into the circuit-breaker.
    struct FailingSink;
    impl crate::sink::EventSink for FailingSink {
        fn write(&mut self, _event: &ExecutionEvent) -> crate::Result<()> {
            Err(crate::XplainitError::InternalError("sink boom".into()))
        }
        fn flush(&mut self) -> crate::Result<()> {
            Ok(())
        }
        fn close(&mut self) -> crate::Result<()> {
            Ok(())
        }
        fn description(&self) -> String {
            "failing".to_string()
        }
    }

    fn make_enter_event() -> ExecutionEvent {
        crate::ExecutionEvent::FunctionEnter {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            location: SourceLocation {
                file: "t".into(),
                line: 1,
                column: 0,
                offset: 0,
            },
            name: "f".into(),
            args: HashMap::new(),
        }
    }

    #[test]
    fn test_pipeline_control_trips_breaker_after_consecutive_errors() {
        // End-to-end: a sink that fails on write drives real framework errors
        // through handle_event -> process_and_sink -> RuntimeControl. After
        // `max_consecutive_errors` consecutive failures the breaker trips and
        // capture is disabled. Fails if `with_control` / the process_and_sink
        // error routing is reverted (the breaker would never see the errors).
        use crate::control::RuntimeControl;

        let filter = Box::new(AcceptAllFilter);
        let processors = ProcessorPipeline::new().add_processor(Box::new(PassThroughProcessor));
        let control = RuntimeControl::new(Config {
            max_consecutive_errors: 3,
            ..Config::new(crate::Language::Python)
        });

        let mut pipeline = EventPipeline::new(filter, processors)
            .add_sink(Box::new(FailingSink))
            .with_control(control.clone());
        let config = Config::new(crate::Language::Python);

        assert!(control.is_capture_enabled());
        assert_eq!(control.consecutive_errors(), 0);

        // Two failing events: errors accumulate but breaker has not tripped.
        let _ = pipeline.handle_event(make_enter_event(), &config);
        let _ = pipeline.handle_event(make_enter_event(), &config);
        assert_eq!(control.consecutive_errors(), 2);
        assert!(!control.is_panic_mode());
        assert!(control.is_capture_enabled());
        assert_eq!(control.times_tripped(), 0);

        // Third consecutive failure trips the breaker and disables capture.
        let _ = pipeline.handle_event(make_enter_event(), &config);
        assert!(control.is_panic_mode(), "breaker should have tripped");
        assert!(!control.is_capture_enabled());
        assert_eq!(control.times_tripped(), 1);
        assert_eq!(control.total_errors(), 3);
    }

    #[test]
    fn test_pipeline_control_success_resets_consecutive_counter() {
        // A clean event through a working sink records a success and resets the
        // consecutive-error counter, so transient failures do not accumulate.
        // Fails if the Ok-branch record_success wiring is reverted.
        use crate::control::RuntimeControl;

        let control = RuntimeControl::new(Config {
            max_consecutive_errors: 3,
            ..Config::new(crate::Language::Python)
        });

        // Pipeline 1: failing sink to rack up two consecutive errors.
        let mut failing = EventPipeline::new(
            Box::new(AcceptAllFilter),
            ProcessorPipeline::new().add_processor(Box::new(PassThroughProcessor)),
        )
        .add_sink(Box::new(FailingSink))
        .with_control(control.clone());
        let config = Config::new(crate::Language::Python);

        let _ = failing.handle_event(make_enter_event(), &config);
        let _ = failing.handle_event(make_enter_event(), &config);
        assert_eq!(control.consecutive_errors(), 2);

        // Pipeline 2: working memory sink sharing the same control. A clean
        // event resets the consecutive counter.
        let mut ok_pipeline = EventPipeline::new(
            Box::new(AcceptAllFilter),
            ProcessorPipeline::new().add_processor(Box::new(PassThroughProcessor)),
        )
        .add_sink(Box::new(MemorySink::new(10)))
        .with_control(control.clone());

        ok_pipeline
            .handle_event(make_enter_event(), &config)
            .unwrap();
        assert_eq!(control.consecutive_errors(), 0);
        assert!(!control.is_panic_mode());
        assert_eq!(control.times_tripped(), 0);
    }
}
