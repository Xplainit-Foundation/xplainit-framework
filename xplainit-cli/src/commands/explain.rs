//! `explain` subcommand: describe a function and, when a trace is supplied,
//! explain that function's recorded execution events.

use anyhow::Result;
use colored::Colorize;
use std::path::Path;
use xplainit_core::{
    AstNode, AstParser, ExecutionEvent, ExplanationGenerator, Language, VerbosityLevel,
};

use super::load_events;

/// Handle `xplainit explain <file::function> [--trace <trace.json>]`.
///
/// Wired-vs-not-wired honesty note:
/// - Static description of the target function is fully wired: when the source
///   file exists on disk, we parse it with the core `AstParser` (Tree-sitter)
///   and report the function's location and signature line.
/// - Trace-driven explanation is fully wired: when `--trace` points at a JSON
///   event log, we filter events belonging to the target function and render
///   them with the core `ExplanationGenerator`.
/// - Attaching to a *live* running process to explain it on the fly is NOT
///   wired from this CLI (it needs the per-language runtime hooks). When no
///   source file and no trace are available we say so explicitly rather than
///   pretending we inspected a live runtime.
pub fn handle(target: &str, trace: Option<&str>) -> Result<()> {
    let (file, function) = parse_target(target)?;
    let output = explain_target(&file, &function, trace)?;
    print!("{output}");
    Ok(())
}

/// Split `file::function` on the LAST `::` so paths containing `::` still work.
fn parse_target(target: &str) -> Result<(String, String)> {
    match target.rsplit_once("::") {
        Some((file, function)) if !file.is_empty() && !function.is_empty() => {
            Ok((file.to_string(), function.to_string()))
        }
        _ => anyhow::bail!("target must be in the form `file::function` (got '{target}')"),
    }
}

/// Detect a `Language` from a file extension, defaulting to Python which the
/// AST parser supports.
fn language_from_path(path: &str) -> Language {
    match Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "py" => Language::Python,
        "js" | "mjs" | "cjs" => Language::JavaScript,
        "rs" => Language::Rust,
        "c" | "h" => Language::C,
        "cpp" | "cc" | "cxx" | "hpp" => Language::Cpp,
        "java" => Language::Java,
        "go" => Language::Go,
        _ => Language::Python,
    }
}

pub(crate) fn explain_target(file: &str, function: &str, trace: Option<&str>) -> Result<String> {
    let mut out = String::new();
    out.push_str(&format!(
        "{} {} {} {}\n",
        "Explaining".bold(),
        function.cyan().bold(),
        "in".bold(),
        file
    ));

    let mut produced_something = false;

    // ----- Static AST description (wired when the source file exists) -----
    if Path::new(file).exists() {
        match describe_from_source(file, function) {
            Ok(Some(desc)) => {
                out.push('\n');
                out.push_str(&desc);
                produced_something = true;
            }
            Ok(None) => {
                out.push_str(&format!(
                    "\n{}\n",
                    format!("Parsed {file} but no function named '{function}' was found.").yellow()
                ));
                produced_something = true;
            }
            Err(e) => {
                out.push_str(&format!(
                    "\n{} {e}\n",
                    "Could not parse source for a static description:".yellow()
                ));
            }
        }
    }

    // ----- Trace-driven explanation (wired when a trace JSON is given) -----
    if let Some(trace_path) = trace {
        let events = load_events(trace_path)?;
        let explained = explain_from_trace(&events, function);
        out.push('\n');
        out.push_str(&explained);
        produced_something = true;
    }

    // ----- Nothing to work with: be honest about the live-runtime gap -----
    if !produced_something {
        out.push('\n');
        out.push_str(&format!(
            "{}\n",
            "No source file on disk and no --trace provided.".yellow()
        ));
        out.push_str(
            "Live attach-and-explain of a running process is not wired into this CLI.\n\
             To explain this function, either:\n  \
             - pass a source path that exists so it can be parsed statically, or\n  \
             - capture a trace with a language binding (Python/Node/Java/C/Go),\n    \
             export it as JSON, and re-run with `--trace <trace.json>`.\n",
        );
    }

    Ok(out)
}

/// Parse the source file and locate a function definition matching `function`.
/// Returns `Ok(None)` when the file parses but the function is not present.
fn describe_from_source(file: &str, function: &str) -> Result<Option<String>> {
    let source = std::fs::read_to_string(file)?;
    let language = language_from_path(file);

    let mut parser = AstParser::new(language);
    parser.parse(source)?;

    let root = match parser.root_node() {
        Some(node) => node,
        None => return Ok(None),
    };

    match find_function_node(&root, function) {
        Some(node) => {
            let mut desc = String::new();
            desc.push_str(&format!("{}\n", "Static analysis:".bold()));
            desc.push_str(&format!("  kind: {}\n", node.kind));
            desc.push_str(&format!(
                "  location: line {}, column {}\n",
                node.start.line + 1,
                node.start.column + 1
            ));
            if let Some(signature) = node.text.lines().next() {
                desc.push_str(&format!("  signature: {}\n", signature.trim()));
            }
            Ok(Some(desc))
        }
        None => Ok(None),
    }
}

/// Depth-first search for a function/method AST node whose recorded name
/// metadata (or first line) mentions `function`.
fn find_function_node(node: &AstNode, function: &str) -> Option<AstNode> {
    let is_function_like = node.kind.contains("function") || node.kind.contains("method");
    if is_function_like {
        let name_matches = node
            .metadata
            .get("name")
            .map(|n| n == function)
            .unwrap_or(false);
        let text_matches = node
            .text
            .lines()
            .next()
            .map(|l| l.contains(function))
            .unwrap_or(false);
        if name_matches || text_matches {
            return Some(node.clone());
        }
    }

    for child in &node.children {
        if let Some(found) = find_function_node(child, function) {
            return Some(found);
        }
    }
    None
}

/// Filter events belonging to `function` and render them with the core
/// `ExplanationGenerator`.
fn explain_from_trace(events: &[ExecutionEvent], function: &str) -> String {
    let generator = ExplanationGenerator::new(VerbosityLevel::Detailed);

    let matched: Vec<&ExecutionEvent> = events
        .iter()
        .filter(|e| event_mentions_function(e, function))
        .collect();

    let mut out = String::new();
    out.push_str(&format!("{}\n", "Trace explanation:".bold()));

    if matched.is_empty() {
        out.push_str(&format!(
            "  {}\n",
            format!("No trace events reference function '{function}'.").yellow()
        ));
        return out;
    }

    for event in matched {
        for line in generator.explain(event).lines() {
            out.push_str(&format!("  {line}\n"));
        }
    }

    out
}

/// Decide whether an event references the given function name.
fn event_mentions_function(event: &ExecutionEvent, function: &str) -> bool {
    match event {
        ExecutionEvent::FunctionEnter { name, .. } | ExecutionEvent::FunctionExit { name, .. } => {
            name == function
        }
        ExecutionEvent::StackOverflow { function: f, .. } => f == function,
        // For other events, fall back to matching the source file's function
        // context via the location file name is not reliable, so only the
        // explicit function-bearing variants are matched.
        _ => false,
    }
}
