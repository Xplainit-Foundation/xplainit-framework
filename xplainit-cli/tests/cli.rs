//! Integration tests that drive the built `xplainit` binary end-to-end.
//!
//! `assert_cmd` is not available in this environment, so the binary is driven
//! directly via `std::process::Command` using the `CARGO_BIN_EXE_xplainit`
//! path that Cargo provides to integration tests. Fixtures are written into a
//! unique per-test directory under the system temp dir and cleaned up after.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_xplainit");

const SAMPLE_TRACE: &str = r#"[
  {
    "FunctionEnter": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "divide",
      "args": {},
      "location": {"file": "app.py", "line": 10, "column": 1, "offset": 0},
      "timestamp": "2024-01-01T00:00:00Z"
    }
  },
  {
    "DivisionByZero": {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "numerator": {"Integer": 10},
      "denominator_var": "x",
      "location": {"file": "app.py", "line": 12, "column": 5, "offset": 0},
      "timestamp": "2024-01-01T00:00:01Z"
    }
  }
]"#;

/// Create a unique temp directory for a test and return its path.
fn temp_dir(tag: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let unique = format!(
        "xplainit-cli-test-{tag}-{}-{:?}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    dir.push(unique);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn write_trace(dir: &Path) -> PathBuf {
    let path = dir.join("trace.json");
    fs::write(&path, SAMPLE_TRACE).expect("write trace fixture");
    path
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(BIN).args(args).output().expect("run xplainit")
}

#[test]
fn help_lists_all_subcommands() {
    let out = run(&["--help"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    for sub in ["run", "explain", "analyze", "report"] {
        assert!(stdout.contains(sub), "help missing subcommand: {sub}");
    }
}

#[test]
fn analyze_reports_error() {
    let dir = temp_dir("analyze");
    let trace = write_trace(&dir);
    let out = run(&["analyze", trace.to_str().unwrap()]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("found 1 error(s)"));
    assert!(stdout.contains("division_by_zero"));
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn report_json_is_valid() {
    let dir = temp_dir("report-json");
    let trace = write_trace(&dir);
    let out = run(&["report", "--format", "json", trace.to_str().unwrap()]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert!(value.is_array());
    assert_eq!(value.as_array().unwrap().len(), 2);
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn report_html_is_document() {
    let dir = temp_dir("report-html");
    let trace = write_trace(&dir);
    let out = run(&["report", "--format", "html", trace.to_str().unwrap()]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("<!DOCTYPE html>"));
    assert!(stdout.contains("</html>"));
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn analyze_missing_file_fails() {
    let out = run(&["analyze", "/no/such/file-xplainit-test.json"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    // Path validation rejects the missing file with a clear, actionable error
    // before it is opened (see xplainit_core::validate_input_path).
    assert!(stderr.contains("does not exist"));
}

#[test]
fn run_on_source_reports_not_wired() {
    let dir = temp_dir("run-src");
    let src = dir.join("prog.py");
    fs::write(&src, "def main():\n    pass\n").unwrap();
    let out = run(&["run", src.to_str().unwrap()]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("not wired"));
    fs::remove_dir_all(&dir).ok();
}

// ===== FEAT-004: additional realistic end-to-end CLI flows =====

/// A trace whose FunctionEnter carries a secret-like `password` argument. Used
/// to prove the CLI redacts secrets before rendering (see report command).
const SECRET_TRACE: &str = r#"[
  {
    "FunctionEnter": {
      "id": "550e8400-e29b-41d4-a716-4466554400aa",
      "name": "login",
      "args": {"password": {"String": "hunter2super"}},
      "location": {"file": "auth.py", "line": 3, "column": 1, "offset": 0},
      "timestamp": "2024-01-01T00:00:00Z"
    }
  }
]"#;

fn write_named(dir: &Path, name: &str, contents: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, contents).expect("write fixture");
    path
}

#[test]
fn report_text_renders_events() {
    // Realistic flow: load trace JSON -> render as text -> assert the human
    // explanation mentions the function. Fails if the report/text formatter
    // path is broken.
    let dir = temp_dir("report-text");
    let trace = write_trace(&dir);
    let out = run(&["report", "--format", "text", trace.to_str().unwrap()]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("divide"),
        "text report missing function name"
    );
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn report_markdown_is_document() {
    let dir = temp_dir("report-md");
    let trace = write_trace(&dir);
    let out = run(&["report", "--format", "markdown", trace.to_str().unwrap()]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("# Xplainit Execution Trace"));
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn report_secret_argument_is_redacted() {
    // End-to-end: a secret-valued argument must NOT appear in rendered output;
    // the redaction placeholder must appear instead. Fails if the CLI stops
    // redacting before it renders.
    let dir = temp_dir("report-secret");
    let trace = write_named(&dir, "secret.json", SECRET_TRACE);
    let out = run(&["report", "--format", "json", trace.to_str().unwrap()]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("hunter2super"),
        "secret value leaked in report output: {stdout}"
    );
    assert!(
        stdout.contains("<redacted>"),
        "redaction placeholder missing from report output: {stdout}"
    );
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn report_unknown_format_fails() {
    let dir = temp_dir("report-bad-fmt");
    let trace = write_trace(&dir);
    let out = run(&["report", "--format", "bogus", trace.to_str().unwrap()]);
    assert!(!out.status.success(), "unknown format must be an error");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn analyze_clean_trace_reports_no_errors() {
    // A trace with only a normal event must report zero errors.
    let dir = temp_dir("analyze-clean");
    let clean = r#"[
      {"FunctionEnter": {
        "id": "550e8400-e29b-41d4-a716-4466554400bb",
        "name": "ok", "args": {},
        "location": {"file": "app.py", "line": 1, "column": 0, "offset": 0},
        "timestamp": "2024-01-01T00:00:00Z"}}
    ]"#;
    let trace = write_named(&dir, "clean.json", clean);
    let out = run(&["analyze", trace.to_str().unwrap()]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("found 0 error(s)") || stdout.to_lowercase().contains("no error"),
        "clean analyze output unexpected: {stdout}"
    );
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn report_malformed_json_fails_gracefully() {
    // A malformed trace file must fail with a non-zero exit, not panic.
    let dir = temp_dir("report-malformed");
    let trace = write_named(&dir, "bad.json", "{ this is not valid json ]");
    let out = run(&["report", "--format", "json", trace.to_str().unwrap()]);
    assert!(!out.status.success(), "malformed JSON must fail");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn analyze_empty_path_is_rejected() {
    // An empty path argument must be rejected by input validation.
    let out = run(&["analyze", ""]);
    assert!(!out.status.success());
}
