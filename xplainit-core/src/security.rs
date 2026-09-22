//! Security hardening helpers shared across the CLI, dashboard and bindings.
//!
//! Two concerns live here:
//!
//! 1. **Input-path validation** ([`validate_input_path`]). User-supplied trace
//!    and log file paths flow in from CLI arguments and the dashboard. Before
//!    the framework opens such a path we reject clearly hostile or nonsensical
//!    input (empty strings, embedded NUL bytes) and confirm the target is a
//!    real, readable file rather than a directory or a missing path. This turns
//!    a would-be panic or confusing downstream error into a clear, actionable
//!    message while still accepting any legitimate relative or absolute path
//!    the user intends to load.
//!
//! 2. **Secret/PII redaction** ([`is_redaction_key`], [`redact_named_value`],
//!    [`DEFAULT_REDACTION_PATTERNS`], [`REDACTED_PLACEHOLDER`]). Captured
//!    runtime values are keyed by variable/argument names. When a key looks
//!    like a secret (password, token, api_key, ...), the value is replaced with
//!    a placeholder before it can leave the process through console output,
//!    JSON serialization or dashboard payloads.

use crate::error::{Result, XplainitError};
use crate::events::Value;
use std::path::Path;

/// Placeholder substituted for values whose key matches a redaction pattern.
pub const REDACTED_PLACEHOLDER: &str = "<redacted>";

/// Default set of case-insensitive substrings that mark a key as secret-like.
///
/// These are matched as substrings so keys such as `db_password`,
/// `user_token` or `apikey` are all covered.
pub const DEFAULT_REDACTION_PATTERNS: &[&str] = &[
    "password",
    "passwd",
    "secret",
    "token",
    "api_key",
    "apikey",
    "authorization",
    "auth",
    "credential",
    "private_key",
];

/// Validate a user-supplied input path before it is opened.
///
/// Rejects, with a clear [`XplainitError`], inputs that are:
/// - empty,
/// - contain an embedded NUL byte (which cannot be a real filesystem path and
///   is a classic injection vector against C-string based APIs),
/// - do not exist, or
/// - exist but are not a regular file (for example a directory).
///
/// Any legitimate relative or absolute path pointing at a real file is
/// accepted unchanged; this is validation, not a lockout.
pub fn validate_input_path(path: &Path) -> Result<()> {
    let as_str = path.to_string_lossy();

    if as_str.is_empty() {
        return Err(XplainitError::ConfigError(
            "input path is empty".to_string(),
        ));
    }

    if as_str.as_bytes().contains(&0) {
        return Err(XplainitError::ConfigError(
            "input path contains an embedded NUL byte".to_string(),
        ));
    }

    if !path.exists() {
        return Err(XplainitError::IoError(format!(
            "input path does not exist: {as_str}"
        )));
    }

    if !path.is_file() {
        return Err(XplainitError::IoError(format!(
            "input path is not a regular file: {as_str}"
        )));
    }

    Ok(())
}

/// Return true when `key` matches any of the supplied redaction `patterns`
/// (case-insensitive substring match).
pub fn is_redaction_key(key: &str, patterns: &[String]) -> bool {
    let key_lower = key.to_lowercase();
    patterns
        .iter()
        .any(|p| key_lower.contains(&p.to_lowercase()))
}

/// If `key` matches a redaction pattern, return the redacted placeholder value;
/// otherwise return `value` unchanged.
///
/// This is the single choke point used by the formatter/serialization paths so
/// bindings and the CLI redact consistently without duplicating the logic.
pub fn redact_named_value(key: &str, value: &Value, patterns: &[String]) -> Value {
    if is_redaction_key(key, patterns) {
        Value::String(REDACTED_PLACEHOLDER.to_string())
    } else {
        value.clone()
    }
}

/// Scrub secret-like `key=value` / `key: value` occurrences out of a free-form
/// message string.
///
/// Key-based redaction only reaches values that carry a *name* (arguments,
/// variables, context maps). Error/exception `message` strings are free-form
/// and are a realistic place for a secret to land inline, for example
/// `"auth failed: token=sk-abc123"` or a connection string with an embedded
/// password. This helper scans the message for a redaction pattern immediately
/// followed by a `=`, `:` or `=>` assignment and replaces the *value* token
/// (everything up to the next whitespace, quote, comma or semicolon) with the
/// redacted placeholder, leaving the surrounding prose and the key itself
/// intact so the message stays useful.
///
/// The scan is case-insensitive on the key and never widens the match beyond a
/// single value token, so ordinary prose that merely mentions the word
/// "password" without an assignment is left untouched.
pub fn redact_message(message: &str, patterns: &[String]) -> String {
    if message.is_empty() {
        return message.to_string();
    }

    let lower = message.to_lowercase();
    // Collect (value_start, value_end) byte ranges to redact, scanning left to
    // right so ranges never overlap.
    let mut redactions: Vec<(usize, usize)> = Vec::new();
    let bytes = message.as_bytes();
    let mut search_from = 0usize;

    for pattern in patterns {
        let p = pattern.to_lowercase();
        if p.is_empty() {
            continue;
        }
        let mut from = 0usize;
        while let Some(rel) = lower[from..].find(&p) {
            let key_start = from + rel;
            let key_end = key_start + p.len();
            from = key_end;

            // Skip whitespace between the key and a separator.
            let mut i = key_end;
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                i += 1;
            }
            // Require an assignment separator (`=`, `:`) directly after the key
            // (optionally `=>`), otherwise this is prose, not an assignment.
            if i >= bytes.len() || (bytes[i] != b'=' && bytes[i] != b':') {
                continue;
            }
            i += 1;
            if i < bytes.len() && bytes[i] == b'>' {
                i += 1; // `=>`
            }
            // Skip whitespace and an optional opening quote before the value.
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                i += 1;
            }
            let value_start = i;
            // The value token runs until the next delimiter.
            while i < bytes.len()
                && !matches!(
                    bytes[i],
                    b' ' | b'\t' | b'\n' | b'\r' | b',' | b';' | b'"' | b'\''
                )
            {
                i += 1;
            }
            let value_end = i;
            if value_end > value_start && value_start >= search_from {
                redactions.push((value_start, value_end));
                search_from = value_end;
            }
        }
    }

    if redactions.is_empty() {
        return message.to_string();
    }

    // Apply non-overlapping redactions left to right.
    redactions.sort_by_key(|r| r.0);
    let mut out = String::with_capacity(message.len());
    let mut cursor = 0usize;
    for (start, end) in redactions {
        if start < cursor {
            continue; // overlap guard
        }
        out.push_str(&message[cursor..start]);
        out.push_str(REDACTED_PLACEHOLDER);
        cursor = end;
    }
    out.push_str(&message[cursor..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn default_patterns() -> Vec<String> {
        DEFAULT_REDACTION_PATTERNS
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn rejects_empty_path() {
        let err = validate_input_path(Path::new("")).unwrap_err();
        assert!(err.to_string().to_lowercase().contains("empty"));
    }

    #[test]
    fn rejects_nul_byte_path() {
        let p = format!("trace{}here.json", '\u{0}');
        let err = validate_input_path(Path::new(&p)).unwrap_err();
        assert!(err.to_string().to_lowercase().contains("nul"));
    }

    #[test]
    fn rejects_missing_file() {
        let err = validate_input_path(Path::new("/nonexistent/xplainit/does-not-exist.json"))
            .unwrap_err();
        assert!(err.to_string().to_lowercase().contains("does not exist"));
    }

    #[test]
    fn rejects_directory() {
        let dir = std::env::temp_dir();
        let err = validate_input_path(&dir).unwrap_err();
        assert!(err
            .to_string()
            .to_lowercase()
            .contains("not a regular file"));
    }

    #[test]
    fn accepts_real_temp_file() {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "xplainit-security-test-{}.json",
            std::process::id()
        ));
        {
            let mut f = std::fs::File::create(&path).expect("create temp file");
            writeln!(f, "[]").expect("write temp file");
        }
        let result = validate_input_path(&path);
        let _ = std::fs::remove_file(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn matches_secret_keys_case_insensitively() {
        let patterns = default_patterns();
        assert!(is_redaction_key("password", &patterns));
        assert!(is_redaction_key("PASSWORD", &patterns));
        assert!(is_redaction_key("db_password", &patterns));
        assert!(is_redaction_key("api_key", &patterns));
        assert!(is_redaction_key("API_KEY", &patterns));
        assert!(is_redaction_key("apikey", &patterns));
        assert!(is_redaction_key("user_token", &patterns));
        assert!(is_redaction_key("Authorization", &patterns));
        assert!(is_redaction_key("private_key", &patterns));
    }

    #[test]
    fn does_not_match_normal_keys() {
        let patterns = default_patterns();
        assert!(!is_redaction_key("count", &patterns));
        assert!(!is_redaction_key("name", &patterns));
        assert!(!is_redaction_key("result", &patterns));
    }

    #[test]
    fn redacts_secret_value_but_not_normal() {
        let patterns = default_patterns();
        let secret =
            redact_named_value("password", &Value::String("hunter2".to_string()), &patterns);
        assert_eq!(secret, Value::String(REDACTED_PLACEHOLDER.to_string()));

        let normal = redact_named_value("count", &Value::Integer(7), &patterns);
        assert_eq!(normal, Value::Integer(7));
    }

    #[test]
    fn redact_message_scrubs_inline_secret_assignment() {
        let patterns = default_patterns();
        // token=... in an error message must be scrubbed, prose kept.
        let msg = "auth failed: token=sk-abc123 while calling api";
        let out = redact_message(msg, &patterns);
        assert!(!out.contains("sk-abc123"), "secret value leaked: {out}");
        assert!(out.contains("auth failed"));
        assert!(out.contains("while calling api"));
        assert!(out.contains(REDACTED_PLACEHOLDER));
    }

    #[test]
    fn redact_message_handles_colon_and_quotes() {
        let patterns = default_patterns();
        let msg = r#"connection refused: password: "s3cr3t!", retrying"#;
        let out = redact_message(msg, &patterns);
        assert!(!out.contains("s3cr3t!"), "secret leaked: {out}");
        assert!(out.contains("connection refused"));
        assert!(out.contains("retrying"));
    }

    #[test]
    fn redact_message_leaves_plain_prose_untouched() {
        let patterns = default_patterns();
        // "password" mentioned without an assignment is prose, not a secret.
        let msg = "the password policy was violated";
        let out = redact_message(msg, &patterns);
        assert_eq!(out, msg);
    }

    #[test]
    fn redact_message_empty_is_noop() {
        let patterns = default_patterns();
        assert_eq!(redact_message("", &patterns), "");
    }
}
