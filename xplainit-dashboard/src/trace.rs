//! Trace loading and parsing.
//!
//! A trace file is a JSON array of `xplainit_core::ExecutionEvent` using the
//! same externally tagged serde shape produced by the core engine and read by
//! the CLI (a single key object per event keyed by the variant name, for
//! example `{"FunctionEnter": {...}}`).

use anyhow::{Context, Result};
use std::path::Path;
use xplainit_core::{redact_events, validate_input_path, Config, ExecutionEvent};

/// Read a JSON file and deserialize it into a `Vec<ExecutionEvent>`.
///
/// Provides clear, contextual errors for the common failure modes (file not
/// found, unreadable, malformed JSON).
///
/// The user-supplied `path` is validated with [`validate_input_path`] before
/// being opened, and secret-like named values are redacted (default [`Config`]
/// policy) so they are never served in dashboard payloads.
pub fn load_events(path: &str) -> Result<Vec<ExecutionEvent>> {
    let p = Path::new(path);
    validate_input_path(p).with_context(|| format!("invalid trace path: {path}"))?;

    let contents =
        std::fs::read_to_string(p).with_context(|| format!("failed to read trace file: {path}"))?;

    let events = parse_events(&contents)
        .with_context(|| format!("failed to parse trace events from: {path}"))?;

    let config = Config::default();
    if config.redact_secrets {
        Ok(redact_events(&events, &config.redact_key_patterns))
    } else {
        Ok(events)
    }
}

/// Parse a JSON string into a `Vec<ExecutionEvent>`.
///
/// Kept separate from `load_events` so it can be unit tested without touching
/// the filesystem.
pub fn parse_events(contents: &str) -> Result<Vec<ExecutionEvent>> {
    let events: Vec<ExecutionEvent> =
        serde_json::from_str(contents).context("invalid trace JSON")?;
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FUNCTION_ENTER: &str = r#"{
        "FunctionEnter": {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "main",
            "args": {},
            "location": {"file": "app.py", "line": 1, "column": 1, "offset": 0},
            "timestamp": "2024-01-01T00:00:00Z"
        }
    }"#;

    const DIVISION_BY_ZERO: &str = r#"{
        "DivisionByZero": {
            "id": "550e8400-e29b-41d4-a716-446655440001",
            "numerator": {"Integer": 10},
            "denominator_var": "x",
            "location": {"file": "app.py", "line": 12, "column": 5, "offset": 0},
            "timestamp": "2024-01-01T00:00:01Z"
        }
    }"#;

    #[test]
    fn parses_array_of_events() {
        let json = format!("[{FUNCTION_ENTER}, {DIVISION_BY_ZERO}]");
        let events = parse_events(&json).expect("parse");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type(), "function_enter");
        assert_eq!(events[1].event_type(), "division_by_zero");
        assert!(events[1].is_error());
    }

    #[test]
    fn parses_empty_array() {
        let events = parse_events("[]").expect("parse empty");
        assert!(events.is_empty());
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(parse_events("not json").is_err());
    }
}
