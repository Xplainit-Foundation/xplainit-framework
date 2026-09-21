//! Subcommand handlers for the Xplainit CLI.

pub mod analyze;
pub mod explain;
pub mod report;
pub mod run;

use anyhow::{Context, Result};
use std::path::Path;
use xplainit_core::ExecutionEvent;

/// Read a JSON file and deserialize it into a `Vec<ExecutionEvent>`.
///
/// Provides clear, contextual errors for the common failure modes
/// (file not found, unreadable, malformed JSON) so the CLI exits with a
/// non-zero status and an actionable message.
pub(crate) fn load_events(path: &str) -> Result<Vec<ExecutionEvent>> {
    let p = Path::new(path);
    if !p.exists() {
        anyhow::bail!("file not found: {path}");
    }

    let contents =
        std::fs::read_to_string(p).with_context(|| format!("failed to read file: {path}"))?;

    let events: Vec<ExecutionEvent> = serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse JSON events from: {path}"))?;

    Ok(events)
}
