//! Subcommand handlers for the Xplainit CLI.

pub mod analyze;
pub mod explain;
pub mod report;
pub mod run;

use anyhow::{Context, Result};
use std::path::Path;
use xplainit_core::{redact_events, validate_input_path, Config, ExecutionEvent};

/// Read a JSON file and deserialize it into a `Vec<ExecutionEvent>`.
///
/// Provides clear, contextual errors for the common failure modes
/// (file not found, unreadable, malformed JSON) so the CLI exits with a
/// non-zero status and an actionable message.
///
/// The user-supplied `path` is validated with [`validate_input_path`] before
/// it is opened (rejecting empty/NUL-byte/missing/non-file inputs), and the
/// loaded events are passed through secret redaction using the default
/// [`Config`] policy so secret-like values never reach console/JSON output.
pub(crate) fn load_events(path: &str) -> Result<Vec<ExecutionEvent>> {
    let p = Path::new(path);
    validate_input_path(p).with_context(|| format!("invalid input path: {path}"))?;

    let contents =
        std::fs::read_to_string(p).with_context(|| format!("failed to read file: {path}"))?;

    let events: Vec<ExecutionEvent> = serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse JSON events from: {path}"))?;

    let config = Config::default();
    if config.redact_secrets {
        Ok(redact_events(&events, &config.redact_key_patterns))
    } else {
        Ok(events)
    }
}
