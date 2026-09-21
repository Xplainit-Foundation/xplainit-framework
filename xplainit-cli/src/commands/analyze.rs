//! `analyze` subcommand: run the ErrorExplainer over a JSON event log.

use anyhow::Result;
use colored::Colorize;
use xplainit_core::{ErrorAnalysis, ErrorExplainer, ErrorSeverity, ExecutionEvent};

use super::load_events;

/// Handle `xplainit analyze <log.json>`.
///
/// Reads the JSON log, feeds every event through `ErrorExplainer` for context,
/// then prints a human-readable, colored analysis for each error event.
pub fn handle(log: &str) -> Result<()> {
    let events = load_events(log)?;
    let report = analyze_events(&events);
    print!("{report}");
    Ok(())
}

/// Build the analyze report as a string. Kept separate from `handle` so it can
/// be unit-tested without touching the filesystem or stdout.
pub(crate) fn analyze_events(events: &[ExecutionEvent]) -> String {
    // Feed all events into the explainer so it can use surrounding context
    // (variable history, recent calls, etc.) when analyzing errors.
    let mut explainer = ErrorExplainer::new();
    for event in events {
        explainer.track_event(event.clone());
    }

    let analyses: Vec<ErrorAnalysis> = events
        .iter()
        .filter(|e| e.is_error())
        .filter_map(|e| explainer.analyze(e))
        .collect();

    let mut out = String::new();

    out.push_str(&format!(
        "{}\n",
        format!(
            "Analyzed {} event(s), found {} error(s).",
            events.len(),
            analyses.len()
        )
        .bold()
    ));

    if analyses.is_empty() {
        out.push_str(&format!(
            "{}\n",
            "No error events detected in this log.".green()
        ));
        return out;
    }

    for (i, analysis) in analyses.iter().enumerate() {
        out.push('\n');
        out.push_str(&format_analysis(i + 1, analysis));
    }

    out
}

fn severity_label(severity: ErrorSeverity) -> colored::ColoredString {
    match severity {
        ErrorSeverity::Warning => "WARNING".yellow().bold(),
        ErrorSeverity::Error => "ERROR".red().bold(),
        ErrorSeverity::Critical => "CRITICAL".red().bold(),
        ErrorSeverity::Fatal => "FATAL".bright_red().bold(),
    }
}

fn format_analysis(index: usize, analysis: &ErrorAnalysis) -> String {
    let mut out = String::new();
    let location = analysis.event.location();

    out.push_str(&format!(
        "{} {} [{}] at {}:{}\n",
        format!("#{index}").dimmed(),
        severity_label(analysis.severity),
        format!("{:?}", analysis.category).cyan(),
        location.file,
        location.line
    ));

    out.push_str(&format!(
        "  {} {}\n",
        "Event:".bold(),
        analysis.event.event_type()
    ));
    out.push_str(&format!(
        "  {} {}\n",
        "Root cause:".bold(),
        analysis.root_cause
    ));

    push_list(&mut out, "Leading events", &analysis.leading_events);
    push_list(&mut out, "Fix suggestions", &analysis.fix_suggestions);
    push_list(&mut out, "Prevention tips", &analysis.prevention_tips);
    push_list(&mut out, "Resources", &analysis.resources);
    push_list(&mut out, "Related errors", &analysis.related_errors);

    out
}

fn push_list(out: &mut String, title: &str, items: &[String]) {
    if items.is_empty() {
        return;
    }
    out.push_str(&format!("  {}\n", format!("{title}:").bold()));
    for item in items {
        out.push_str(&format!("    - {item}\n"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Build fixture events by deserializing JSON. This avoids depending on
    // `uuid`/`chrono` directly in the CLI crate just for test construction.
    const FUNCTION_ENTER: &str = r#"{
        "FunctionEnter": {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "divide",
            "args": {},
            "location": {"file": "app.py", "line": 10, "column": 1, "offset": 0},
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

    fn event(json: &str) -> ExecutionEvent {
        serde_json::from_str(json).expect("valid fixture event")
    }

    #[test]
    fn reports_error_events() {
        colored::control::set_override(false);
        let events = vec![event(FUNCTION_ENTER), event(DIVISION_BY_ZERO)];
        let report = analyze_events(&events);
        assert!(report.contains("Analyzed 2 event(s), found 1 error(s)."));
        assert!(report.contains("division_by_zero"));
        assert!(report.contains("Division by zero"));
        assert!(report.contains("Fix suggestions"));
    }

    #[test]
    fn reports_no_errors_when_none() {
        colored::control::set_override(false);
        let events = vec![event(FUNCTION_ENTER)];
        let report = analyze_events(&events);
        assert!(report.contains("No error events detected"));
    }
}
