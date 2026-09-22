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
/// loaded events are passed through secret redaction using the environment
/// [`Config`] policy ([`Config::from_env`]) so `XPLAINIT_*` settings and any
/// customized `redact_key_patterns` actually take effect on this load path,
/// while still redacting secret-like values before they reach console/JSON
/// output.
pub(crate) fn load_events(path: &str) -> Result<Vec<ExecutionEvent>> {
    load_events_with_config(path, &Config::from_env())
}

/// Load events using an explicit [`Config`] policy.
///
/// Kept separate so the redaction policy can be injected in tests without
/// depending on process-wide environment variables.
pub(crate) fn load_events_with_config(path: &str, config: &Config) -> Result<Vec<ExecutionEvent>> {
    let p = Path::new(path);
    validate_input_path(p).with_context(|| format!("invalid input path: {path}"))?;

    let contents =
        std::fs::read_to_string(p).with_context(|| format!("failed to read file: {path}"))?;

    let events: Vec<ExecutionEvent> = serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse JSON events from: {path}"))?;

    if config.redact_secrets {
        Ok(redact_events(&events, &config.redact_key_patterns))
    } else {
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use xplainit_core::Language;

    fn write_temp(name: &str, contents: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "xplainit-cli-load-{}-{}.json",
            std::process::id(),
            name
        ));
        let mut f = std::fs::File::create(&path).expect("create temp file");
        f.write_all(contents.as_bytes()).expect("write temp file");
        path
    }

    const CUSTOM_SECRET_EVENT: &str = r#"[{
        "FunctionEnter": {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "login",
            "args": {"session_cookie": {"String": "abc123secret"}},
            "location": {"file": "app.py", "line": 1, "column": 1, "offset": 0},
            "timestamp": "2024-01-01T00:00:00Z"
        }
    }]"#;

    #[test]
    fn honors_custom_redact_key_patterns_on_load() {
        // "session_cookie" is NOT in the default patterns, so with the default
        // config it survives. A caller-provided config that lists "cookie" as a
        // pattern must redact it on the load path. This guards against the
        // regression where load_events bound a hardcoded Config::default() and
        // dropped configured patterns. Uses an explicit config (not env) to
        // avoid test env pollution.
        let path = write_temp("custom-pattern", CUSTOM_SECRET_EVENT);

        // Default policy: not redacted (proves the value is present to begin with).
        let default_cfg = Config::new(Language::Python);
        let events = load_events_with_config(path.to_str().unwrap(), &default_cfg).unwrap();
        let json = serde_json::to_string(&events).unwrap();
        assert!(
            json.contains("abc123secret"),
            "value should be present under default patterns: {json}"
        );

        // Custom policy adds "cookie": now the value must be redacted.
        let custom_cfg = Config {
            redact_key_patterns: vec!["cookie".to_string()],
            ..Config::new(Language::Python)
        };
        let events = load_events_with_config(path.to_str().unwrap(), &custom_cfg).unwrap();
        let json = serde_json::to_string(&events).unwrap();
        let _ = std::fs::remove_file(&path);
        assert!(
            !json.contains("abc123secret"),
            "custom pattern was not honored on load: {json}"
        );
    }
}
