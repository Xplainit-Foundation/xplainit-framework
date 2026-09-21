//! Trace loading and parsing.
//!
//! A trace file is a JSON array of `xplainit_core::ExecutionEvent` using the
//! same externally tagged serde shape produced by the core engine and read by
//! the CLI (a single key object per event keyed by the variant name, for
//! example `{"FunctionEnter": {...}}`).

use anyhow::{Context, Result};
use std::path::Path;
use xplainit_core::ExecutionEvent;

/// Read a JSON file and deserialize it into a `Vec<ExecutionEvent>`.
///
/// Provides clear, contextual errors for the common failure modes (file not
/// found, unreadable, malformed JSON).
pub fn load_events(path: &str) -> Result<Vec<ExecutionEvent>> {
    let p = Path::new(path);
    if !p.exists() {
        anyhow::bail!("trace file not found: {path}");
    }

    let contents =
        std::fs::read_to_string(p).with_context(|| format!("failed to read trace file: {path}"))?;

    parse_events(&contents).with_context(|| format!("failed to parse trace events from: {path}"))
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
