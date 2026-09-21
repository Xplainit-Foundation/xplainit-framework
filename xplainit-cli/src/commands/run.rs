//! `run` subcommand: entry point for live tracing of a program.

use anyhow::Result;
use colored::Colorize;
use std::path::Path;
use xplainit_core::Language;

use super::load_events;
use super::report;

/// Handle `xplainit run <file>`.
///
/// Honesty note about what is and is not wired to a live runtime:
/// Full live tracing of an arbitrary external program requires the per-language
/// runtime hooks (PyO3 sys.settrace, Node inspector, JVMTI agent, C/Go FFI
/// callbacks). Those hooks live in the binding crates and are NOT invocable
/// from this standalone CLI in the current environment. So:
///
/// - If given a `.json` file, we treat it as an already-captured trace and
///   render it (a genuinely working path), delegating to `report`.
/// - For any source file we can identify by extension, we DO NOT print a fake
///   "traced successfully" message. Instead we explain exactly which binding
///   is required and how to produce a trace this CLI can consume.
pub fn handle(file: &str) -> Result<()> {
    let path = Path::new(file);
    if !path.exists() {
        anyhow::bail!("file not found: {file}");
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Genuinely-working path: a pre-captured JSON trace can be rendered now.
    if ext == "json" {
        // Validate it really is a trace before claiming anything.
        let events = load_events(file)?;
        println!(
            "{}",
            format!(
                "Detected a captured trace ({} event(s)). Rendering it:",
                events.len()
            )
            .green()
        );
        return report::handle(file, "text");
    }

    // Not-yet-wired path: identify the language and tell the user exactly what
    // is required. We deliberately do not print success we did not achieve.
    let language = detect_language(&ext);
    print_live_tracing_guidance(file, language);
    Ok(())
}

fn detect_language(ext: &str) -> Option<Language> {
    match ext {
        "py" => Some(Language::Python),
        "js" | "mjs" | "cjs" => Some(Language::JavaScript),
        "rs" => Some(Language::Rust),
        "c" | "h" => Some(Language::C),
        "cpp" | "cc" | "cxx" | "hpp" => Some(Language::Cpp),
        "java" => Some(Language::Java),
        "go" => Some(Language::Go),
        _ => None,
    }
}

fn print_live_tracing_guidance(file: &str, language: Option<Language>) {
    match language {
        Some(lang) => {
            println!(
                "{}",
                format!("Recognized {} source: {file}", lang.as_str()).bold()
            );
            println!(
                "{}",
                "Live tracing is not wired into this standalone CLI.".yellow()
            );
            let binding = match lang {
                Language::Python => "the xplainit Python module (`import xplainit`)",
                Language::JavaScript => "the xplainit Node binding",
                Language::Java => "the xplainit JVMTI agent",
                Language::C | Language::Cpp => "the xplainit C FFI recorder",
                Language::Go => "the xplainit Go (CGO) recorder",
                Language::Rust => "a xplainit-core embedding in your program",
            };
            println!("To trace this program, run it under {binding},");
            println!("export the captured events as a JSON array of ExecutionEvent values,");
            println!("then analyze or render them with:");
            println!("  xplainit report <trace.json> --format text");
            println!("  xplainit analyze <trace.json>");
        }
        None => {
            println!("{}", format!("Unrecognized file type for: {file}").yellow());
            println!(
                "Supported inputs: a captured trace `.json`, or a source file \
                 (.py/.js/.rs/.c/.cpp/.java/.go) to be traced by its language binding."
            );
        }
    }
}
