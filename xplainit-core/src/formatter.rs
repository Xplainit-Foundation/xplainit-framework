//! Output formatting system for multiple formats (JSON, HTML, Markdown)
use crate::events::*;
use crate::explainer::{ExplanationGenerator, VerbosityLevel};
use crate::config::OutputFormat;
use std::fmt::Write;

/// Formatter trait for different output formats
pub trait OutputFormatter: Send + Sync {
    /// Format a single event
    fn format_event(&self, event: &ExecutionEvent) -> String;
    
    /// Format multiple events
    fn format_events(&self, events: &[ExecutionEvent]) -> String {
        events.iter()
            .map(|e| self.format_event(e))
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Get format name
    fn format_name(&self) -> &'static str;
    
    /// Start of document/stream
    fn header(&self) -> String {
        String::new()
    }
    
    /// End of document/stream
    fn footer(&self) -> String {
        String::new()
    }
}

/// Plain text formatter
pub struct TextFormatter {
    explainer: ExplanationGenerator,
    show_timestamp: bool,
    show_ids: bool,
}

impl TextFormatter {
    pub fn new(verbosity: VerbosityLevel) -> Self {
        Self {
            explainer: ExplanationGenerator::new(verbosity)
                .with_timestamps(false)
                .with_ids(false),
            show_timestamp: false,
            show_ids: false,
        }
    }
    
    pub fn with_timestamps(mut self, show: bool) -> Self {
        self.show_timestamp = show;
        self.explainer = self.explainer.with_timestamps(show);
        self
    }
    
    pub fn with_ids(mut self, show: bool) -> Self {
        self.show_ids = show;
        self.explainer = self.explainer.with_ids(show);
        self
    }
}

impl OutputFormatter for TextFormatter {
    fn format_event(&self, event: &ExecutionEvent) -> String {
        self.explainer.explain(event)
    }
    
    fn format_name(&self) -> &'static str {
        "text"
    }
}

/// JSON formatter
pub struct JsonFormatter {
    pretty: bool,
    include_metadata: bool,
}

impl JsonFormatter {
    pub fn new(pretty: bool) -> Self {
        Self {
            pretty,
            include_metadata: true,
        }
    }
    
    pub fn with_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }
}

impl OutputFormatter for JsonFormatter {
    fn format_event(&self, event: &ExecutionEvent) -> String {
        if self.pretty {
            serde_json::to_string_pretty(event).unwrap_or_else(|_| "{}".to_string())
        } else {
            serde_json::to_string(event).unwrap_or_else(|_| "{}".to_string())
        }
    }
    
    fn format_events(&self, events: &[ExecutionEvent]) -> String {
        if self.pretty {
            serde_json::to_string_pretty(events).unwrap_or_else(|_| "[]".to_string())
        } else {
            serde_json::to_string(events).unwrap_or_else(|_| "[]".to_string())
        }
    }
    
    fn format_name(&self) -> &'static str {
        "json"
    }
    
    fn header(&self) -> String {
        if self.pretty {
            "[\n".to_string()
        } else {
            "[".to_string()
        }
    }
    
    fn footer(&self) -> String {
        if self.pretty {
            "\n]".to_string()
        } else {
            "]".to_string()
        }
    }
}

/// HTML formatter
pub struct HtmlFormatter {
    include_css: bool,
    dark_mode: bool,
}

impl HtmlFormatter {
    pub fn new() -> Self {
        Self {
            include_css: true,
            dark_mode: false,
        }
    }
    
    pub fn with_css(mut self, include: bool) -> Self {
        self.include_css = include;
        self
    }
    
    pub fn with_dark_mode(mut self, dark: bool) -> Self {
        self.dark_mode = dark;
        self
    }
    
    fn get_css(&self) -> &'static str {
        if self.dark_mode {
            include_str!("../resources/dark.css")
        } else {
            include_str!("../resources/light.css")
        }
    }
}

impl OutputFormatter for HtmlFormatter {
    fn format_event(&self, event: &ExecutionEvent) -> String {
        let mut html = String::new();
        
        let class = if event.is_error() { "event error" } else { "event" };
        let _ = writeln!(html, "<div class=\"{}\">", class);
        
        // Event header
        let _ = writeln!(html, "  <div class=\"event-header\">");
        let _ = writeln!(html, "    <span class=\"event-type\">{}</span>", event.event_type());
        let _ = writeln!(html, "    <span class=\"timestamp\">{}</span>", event.timestamp().format("%H:%M:%S%.3f"));
        let _ = writeln!(html, "  </div>");
        
        // Event body
        let _ = writeln!(html, "  <div class=\"event-body\">");
        match event {
            ExecutionEvent::FunctionEnter { name, location, .. } => {
                let _ = writeln!(html, "    <p><strong>Calling function:</strong> {}</p>", name);
                let _ = writeln!(html, "    <p class=\"location\">{}:{}</p>", location.file, location.line);
            }
            ExecutionEvent::FunctionExit { name, return_value, duration, .. } => {
                let _ = writeln!(html, "    <p><strong>Function returned:</strong> {}</p>", name);
                if let Some(val) = return_value {
                    let _ = writeln!(html, "    <p><strong>Value:</strong> {:?}</p>", val);
                }
                let _ = writeln!(html, "    <p class=\"duration\">Duration: {:.3}ms</p>", duration.as_secs_f64() * 1000.0);
            }
            ExecutionEvent::Exception { error_type, message, location, .. } => {
                let _ = writeln!(html, "    <p class=\"error-type\">❌ {}</p>", error_type);
                let _ = writeln!(html, "    <p class=\"error-message\">{}</p>", message);
                let _ = writeln!(html, "    <p class=\"location\">{}:{}</p>", location.file, location.line);
            }
            ExecutionEvent::DivisionByZero { numerator, location, .. } => {
                let _ = writeln!(html, "    <p class=\"error-type\">❌ Division by Zero</p>");
                let _ = writeln!(html, "    <p>Tried to divide {:?} by zero</p>", numerator);
                let _ = writeln!(html, "    <p class=\"location\">{}:{}</p>", location.file, location.line);
            }
            _ => {
                let _ = writeln!(html, "    <p>{:?}</p>", event);
            }
        }
        let _ = writeln!(html, "  </div>");
        let _ = writeln!(html, "</div>");
        
        html
    }
    
    fn format_name(&self) -> &'static str {
        "html"
    }
    
    fn header(&self) -> String {
        let mut header = String::new();
        let _ = writeln!(header, "<!DOCTYPE html>");
        let _ = write!(header, "<html>\n<head>\n");
        let _ = writeln!(header, "  <meta charset=\"UTF-8\">");
        let _ = writeln!(header, "  <title>Xplainit Trace</title>");
        
        if self.include_css {
            let _ = write!(header, "  <style>\n{}\n  </style>\n", self.get_css());
        }
        
        let _ = write!(header, "</head>\n<body>\n");
        let _ = writeln!(header, "<div class=\"container\">");
        let _ = writeln!(header, "  <h1>Xplainit Execution Trace</h1>");
        let _ = writeln!(header, "  <div class=\"events\">");
        
        header
    }
    
    fn footer(&self) -> String {
        let mut footer = String::new();
        let _ = writeln!(footer, "  </div>"); // events
        let _ = writeln!(footer, "</div>"); // container
        let _ = write!(footer, "</body>\n</html>\n");
        footer
    }
}

impl Default for HtmlFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Markdown formatter
pub struct MarkdownFormatter {
    _verbosity: VerbosityLevel,
    include_toc: bool,
}

impl MarkdownFormatter {
    pub fn new(verbosity: VerbosityLevel) -> Self {
        Self {
            _verbosity: verbosity,
            include_toc: false,
        }
    }
    
    pub fn with_toc(mut self, include: bool) -> Self {
        self.include_toc = include;
        self
    }
}

impl OutputFormatter for MarkdownFormatter {
    fn format_event(&self, event: &ExecutionEvent) -> String {
        let mut md = String::new();
        
        // Event header with emoji
        let emoji = if event.is_error() { "❌" } else { "▶️" };
        let _ = writeln!(md, "## {} {}", emoji, event.event_type());
        let _ = writeln!(md);
        
        // Metadata
        let _ = writeln!(md, "**Timestamp:** {}", event.timestamp().format("%H:%M:%S%.3f"));
        let _ = writeln!(md, "**Event ID:** {}", event.id());
        let _ = writeln!(md);
        
        // Event details
        match event {
            ExecutionEvent::FunctionEnter { name, args, location, .. } => {
                let _ = writeln!(md, "**Function:** `{}`", name);
                let _ = writeln!(md, "**Location:** `{}:{}`", location.file, location.line);
                if !args.is_empty() {
                    let _ = writeln!(md, "\n**Arguments:**");
                    for (key, val) in args {
                        let _ = writeln!(md, "- `{}`: `{:?}`", key, val);
                    }
                }
            }
            ExecutionEvent::FunctionExit { name, return_value, duration, .. } => {
                let _ = writeln!(md, "**Function:** `{}`", name);
                let _ = writeln!(md, "**Duration:** {:.3}ms", duration.as_secs_f64() * 1000.0);
                if let Some(val) = return_value {
                    let _ = writeln!(md, "**Return Value:** `{:?}`", val);
                }
            }
            ExecutionEvent::Exception { error_type, message, location, caught, .. } => {
                let status = if *caught { "Caught" } else { "Uncaught" };
                let _ = writeln!(md, "**Error Type:** `{}`", error_type);
                let _ = writeln!(md, "**Status:** {}", status);
                let _ = writeln!(md, "**Message:** {}", message);
                let _ = writeln!(md, "**Location:** `{}:{}`", location.file, location.line);
            }
            _ => {
                let _ = writeln!(md, "```");
                let _ = writeln!(md, "{:?}", event);
                let _ = writeln!(md, "```");
            }
        }
        
        let _ = writeln!(md);
        let _ = writeln!(md, "---");
        let _ = writeln!(md);
        
        md
    }
    
    fn format_name(&self) -> &'static str {
        "markdown"
    }
    
    fn header(&self) -> String {
        let mut header = String::new();
        let _ = writeln!(header, "# Xplainit Execution Trace");
        let _ = writeln!(header);
        let _ = writeln!(header, "Generated at: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
        let _ = writeln!(header);
        
        if self.include_toc {
            let _ = writeln!(header, "## Table of Contents");
            let _ = writeln!(header);
            let _ = writeln!(header, "- [Events](#events)");
            let _ = writeln!(header);
        }
        
        let _ = writeln!(header, "## Events");
        let _ = writeln!(header);
        
        header
    }
}

/// Factory for creating formatters
pub struct FormatterFactory;

impl FormatterFactory {
    pub fn create(format: OutputFormat, verbosity: VerbosityLevel) -> Box<dyn OutputFormatter> {
        match format {
            OutputFormat::Console | OutputFormat::ConsoleColored => {
                Box::new(TextFormatter::new(verbosity))
            }
            OutputFormat::Json => {
                Box::new(JsonFormatter::new(true))
            }
            OutputFormat::Html => {
                Box::new(HtmlFormatter::new())
            }
            OutputFormat::Markdown => {
                Box::new(MarkdownFormatter::new(verbosity))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;
    use std::collections::HashMap;

    #[test]
    fn test_text_formatter() {
        let formatter = TextFormatter::new(VerbosityLevel::Normal);
        
        let event = ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            args: HashMap::new(),
            location: SourceLocation::new("test.py".to_string(), 10, 5),
            timestamp: Utc::now(),
        };
        
        let output = formatter.format_event(&event);
        assert!(output.contains("Calling function test"));
    }
    
    #[test]
    fn test_json_formatter() {
        let formatter = JsonFormatter::new(true);
        
        let event = ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            args: HashMap::new(),
            location: SourceLocation::new("test.py".to_string(), 10, 5),
            timestamp: Utc::now(),
        };
        
        let output = formatter.format_event(&event);
        assert!(output.contains("FunctionEnter"));
        assert!(output.contains("\"name\""));
    }
    
    #[test]
    fn test_html_formatter() {
        let formatter = HtmlFormatter::new();
        
        let event = ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            args: HashMap::new(),
            location: SourceLocation::new("test.py".to_string(), 10, 5),
            timestamp: Utc::now(),
        };
        
        let output = formatter.format_event(&event);
        assert!(output.contains("<div class=\"event\">"));
        assert!(output.contains("test"));
        
        let header = formatter.header();
        assert!(header.contains("<!DOCTYPE html>"));
        
        let footer = formatter.footer();
        assert!(footer.contains("</html>"));
    }
    
    #[test]
    fn test_markdown_formatter() {
        let formatter = MarkdownFormatter::new(VerbosityLevel::Normal);
        
        let event = ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            args: HashMap::new(),
            location: SourceLocation::new("test.py".to_string(), 10, 5),
            timestamp: Utc::now(),
        };
        
        let output = formatter.format_event(&event);
        assert!(output.contains("## ▶️"));
        assert!(output.contains("**Function:**"));
        assert!(output.contains("`test`"));
    }
    
    #[test]
    fn test_formatter_factory() {
        let formatter = FormatterFactory::create(OutputFormat::Json, VerbosityLevel::Normal);
        assert_eq!(formatter.format_name(), "json");
        
        let formatter = FormatterFactory::create(OutputFormat::Html, VerbosityLevel::Normal);
        assert_eq!(formatter.format_name(), "html");
    }
}
