//! Event Sink - Output routing for events
//! 
//! Sinks handle the final destination of events: console, file, network, etc.

use crate::{ExecutionEvent, Result, XplainitError, OutputFormat};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;

/// Trait for event sinks
pub trait EventSink: Send + Sync {
    /// Write an event to the sink
    fn write(&mut self, event: &ExecutionEvent) -> Result<()>;
    
    /// Flush any buffered data
    fn flush(&mut self) -> Result<()>;
    
    /// Close the sink
    fn close(&mut self) -> Result<()>;
    
    /// Get sink description
    fn description(&self) -> String;
}

/// Console sink - writes to stdout/stderr
#[derive(Debug, Clone)]
pub struct ConsoleSink {
    format: OutputFormat,
    use_colors: bool,
    to_stderr: bool,
}

impl ConsoleSink {
    pub fn new(format: OutputFormat) -> Self {
        Self {
            format,
            use_colors: true,
            to_stderr: false,
        }
    }
    
    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.use_colors = use_colors;
        self
    }
    
    pub fn to_stderr(mut self) -> Self {
        self.to_stderr = true;
        self
    }
}

impl EventSink for ConsoleSink {
    fn write(&mut self, event: &ExecutionEvent) -> Result<()> {
        let output = match self.format {
            OutputFormat::Json => {
                serde_json::to_string(event)
                    .map_err(|e| XplainitError::InternalError(e.to_string()))?
            }
            OutputFormat::Console | OutputFormat::ConsoleColored => {
                format!("{:?}", event) // TODO: Implement better formatting
            }
            _ => {
                return Err(XplainitError::InternalError(
                    "Unsupported format for console sink".into()
                ));
            }
        };
        
        if self.to_stderr {
            eprintln!("{}", output);
        } else {
            println!("{}", output);
        }
        
        Ok(())
    }
    
    fn flush(&mut self) -> Result<()> {
        use std::io::{self, Write};
        if self.to_stderr {
            io::stderr().flush()
        } else {
            io::stdout().flush()
        }
        .map_err(|e| XplainitError::IoError(e.to_string()))?;
        Ok(())
    }
    
    fn close(&mut self) -> Result<()> {
        self.flush()
    }
    
    fn description(&self) -> String {
        format!(
            "Console sink (format: {:?}, colors: {}, stderr: {})",
            self.format, self.use_colors, self.to_stderr
        )
    }
}

/// File sink - writes to a file
pub struct FileSink {
    file: Arc<Mutex<File>>,
    path: PathBuf,
    format: OutputFormat,
    buffer_size: usize,
    buffer: Vec<String>,
}

impl FileSink {
    pub fn new(path: PathBuf, format: OutputFormat) -> Result<Self> {
        let file = File::create(&path)
            .map_err(|e| XplainitError::IoError(e.to_string()))?;
        
        Ok(Self {
            file: Arc::new(Mutex::new(file)),
            path,
            format,
            buffer_size: 100,
            buffer: Vec::new(),
        })
    }
    
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }
}

impl EventSink for FileSink {
    fn write(&mut self, event: &ExecutionEvent) -> Result<()> {
        let output = match self.format {
            OutputFormat::Json => {
                serde_json::to_string(event)
                    .map_err(|e| XplainitError::InternalError(e.to_string()))?
            }
            _ => {
                format!("{:?}\n", event)
            }
        };
        
        self.buffer.push(output);
        
        // Flush if buffer is full
        if self.buffer.len() >= self.buffer_size {
            self.flush()?;
        }
        
        Ok(())
    }
    
    fn flush(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        
        let mut file = self.file.lock();
        for line in &self.buffer {
            file.write_all(line.as_bytes())
                .map_err(|e| XplainitError::IoError(e.to_string()))?;
        }
        file.flush()
            .map_err(|e| XplainitError::IoError(e.to_string()))?;
        
        self.buffer.clear();
        Ok(())
    }
    
    fn close(&mut self) -> Result<()> {
        self.flush()
    }
    
    fn description(&self) -> String {
        format!(
            "File sink (path: {:?}, format: {:?}, buffer: {})",
            self.path, self.format, self.buffer_size
        )
    }
}

/// Memory sink - stores events in memory
#[derive(Debug, Clone)]
pub struct MemorySink {
    events: Arc<Mutex<Vec<ExecutionEvent>>>,
    max_events: usize,
}

impl MemorySink {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            max_events,
        }
    }
    
    pub fn get_events(&self) -> Vec<ExecutionEvent> {
        self.events.lock().clone()
    }
    
    pub fn clear(&self) {
        self.events.lock().clear();
    }
}

impl EventSink for MemorySink {
    fn write(&mut self, event: &ExecutionEvent) -> Result<()> {
        let mut events = self.events.lock();
        
        // Limit memory usage
        if events.len() >= self.max_events {
            events.remove(0); // Remove oldest
        }
        
        events.push(event.clone());
        Ok(())
    }
    
    fn flush(&mut self) -> Result<()> {
        // Nothing to flush for memory sink
        Ok(())
    }
    
    fn close(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn description(&self) -> String {
        format!(
            "Memory sink ({}/{} events)",
            self.events.lock().len(),
            self.max_events
        )
    }
}

/// Multi-sink - writes to multiple sinks
pub struct MultiSink {
    sinks: Vec<Box<dyn EventSink>>,
}

impl MultiSink {
    pub fn new() -> Self {
        Self {
            sinks: Vec::new(),
        }
    }
    
    pub fn add_sink(mut self, sink: Box<dyn EventSink>) -> Self {
        self.sinks.push(sink);
        self
    }
}

impl Default for MultiSink {
    fn default() -> Self {
        Self::new()
    }
}

impl EventSink for MultiSink {
    fn write(&mut self, event: &ExecutionEvent) -> Result<()> {
        let mut errors = Vec::new();
        
        for sink in &mut self.sinks {
            if let Err(e) = sink.write(event) {
                errors.push(e);
            }
        }
        
        if !errors.is_empty() {
            return Err(XplainitError::InternalError(
                format!("Failed to write to {} sinks", errors.len())
            ));
        }
        
        Ok(())
    }
    
    fn flush(&mut self) -> Result<()> {
        for sink in &mut self.sinks {
            sink.flush()?;
        }
        Ok(())
    }
    
    fn close(&mut self) -> Result<()> {
        for sink in &mut self.sinks {
            sink.close()?;
        }
        Ok(())
    }
    
    fn description(&self) -> String {
        format!("Multi sink ({} sinks)", self.sinks.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SourceLocation};
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_console_sink() {
        let mut sink = ConsoleSink::new(OutputFormat::Console);
        
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
        
        // Should not panic
        sink.write(&event).unwrap();
        sink.flush().unwrap();
    }

    #[test]
    fn test_memory_sink() {
        let mut sink = MemorySink::new(10);
        
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
        
        sink.write(&event).unwrap();
        
        let events = sink.get_events();
        assert_eq!(events.len(), 1);
        
        sink.clear();
        assert_eq!(sink.get_events().len(), 0);
    }

    #[test]
    fn test_memory_sink_overflow() {
        let mut sink = MemorySink::new(2);
        
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
                value: None,
                var_type: None,
                is_const: false,
            };
            sink.write(&event).unwrap();
        }
        
        // Should only have 2 events (oldest removed)
        assert_eq!(sink.get_events().len(), 2);
    }
}
