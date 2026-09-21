//! `report` subcommand: render a JSON event trace with the core formatters.

use anyhow::Result;
use xplainit_core::{ExecutionEvent, FormatterFactory, OutputFormat, VerbosityLevel};

use super::load_events;

/// Handle `xplainit report <trace.json> --format <fmt>`.
pub fn handle(trace: &str, format: &str) -> Result<()> {
    let events = load_events(trace)?;
    let rendered = render(&events, format)?;
    println!("{rendered}");
    Ok(())
}

/// Map the CLI `--format` string to the core `OutputFormat` enum.
fn parse_format(format: &str) -> Result<OutputFormat> {
    match format.to_lowercase().as_str() {
        "text" | "console" => Ok(OutputFormat::Console),
        "json" => Ok(OutputFormat::Json),
        "html" => Ok(OutputFormat::Html),
        "markdown" | "md" => Ok(OutputFormat::Markdown),
        other => {
            anyhow::bail!("unknown format '{other}'. Valid formats: text, json, html, markdown")
        }
    }
}

/// Render the events using the core `FormatterFactory`. Reuses the existing
/// formatters (Text/Json/Html/Markdown) rather than reimplementing them.
/// Kept separate from `handle` so it can be unit-tested directly.
pub(crate) fn render(events: &[ExecutionEvent], format: &str) -> Result<String> {
    let output_format = parse_format(format)?;
    let formatter = FormatterFactory::create(output_format, VerbosityLevel::Normal);

    // The JSON formatter's `format_events` already serializes the full slice
    // into a single JSON array, so its header/footer (`[` and `]`) must NOT be
    // applied on top or the result would be a doubly-wrapped, invalid array.
    // Text/HTML/Markdown formatters render per-event and rely on the wrapping
    // header/footer for a well-formed document.
    if output_format == OutputFormat::Json {
        return Ok(formatter.format_events(events));
    }

    let mut out = String::new();
    let header = formatter.header();
    if !header.is_empty() {
        out.push_str(&header);
    }
    out.push_str(&formatter.format_events(events));
    let footer = formatter.footer();
    if !footer.is_empty() {
        out.push_str(&footer);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Build fixture events from JSON to avoid a direct uuid/chrono dependency.
    const FUNCTION_ENTER: &str = r#"{
        "FunctionEnter": {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "divide",
            "args": {},
            "location": {"file": "app.py", "line": 10, "column": 1, "offset": 0},
            "timestamp": "2024-01-01T00:00:00Z"
        }
    }"#;

    fn sample_events() -> Vec<ExecutionEvent> {
        vec![serde_json::from_str(FUNCTION_ENTER).expect("valid fixture event")]
    }

    #[test]
    fn json_output_is_valid_array() {
        let events = sample_events();
        let out = render(&events, "json").unwrap();
        // Must round-trip back into events, proving it is a single valid array.
        let parsed: Vec<ExecutionEvent> = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed.len(), 1);
        assert!(out.contains("FunctionEnter"));
    }

    #[test]
    fn html_output_is_wrapped_document() {
        let events = sample_events();
        let out = render(&events, "html").unwrap();
        assert!(out.contains("<!DOCTYPE html>"));
        assert!(out.contains("</html>"));
        assert!(out.contains("divide"));
    }

    #[test]
    fn markdown_output_has_header() {
        let events = sample_events();
        let out = render(&events, "markdown").unwrap();
        assert!(out.contains("# Xplainit Execution Trace"));
        assert!(out.contains("function_enter"));
    }

    #[test]
    fn unknown_format_errors() {
        let events = sample_events();
        assert!(render(&events, "bogus").is_err());
    }
}
