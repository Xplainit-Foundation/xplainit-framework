//! Xplainit command-line interface.
//!
//! This binary wires the `xplainit-core` engine to a set of clap subcommands:
//!
//! - `analyze <log.json>`  - run the ErrorExplainer over a JSON event log.
//! - `report <trace.json>` - render a JSON event trace with the core formatters.
//! - `explain <file::function>` - describe a function via the AST parser and,
//!   when a trace is supplied, explain that function's recorded events.
//! - `run <file>` - entry point for live tracing.
//!
//! Honesty note about what is and is not wired to a live runtime:
//! `analyze`, `report`, and the trace-driven parts of `explain` are fully
//! functional today because they operate on trace/log JSON produced by the
//! language bindings (Python/Node/Java/C/Go). Live instrumentation of an
//! arbitrary external program (`run`, and the "attach to a running process"
//! part of `explain`) requires the per-language runtime hooks that are not
//! invocable from this CLI in the current environment. Those paths therefore
//! print an explicit, actionable message describing what is required instead
//! of pretending a trace was captured.

mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Xplainit: step-by-step code execution explanation from the command line.
#[derive(Parser)]
#[command(
    name = "xplainit",
    author,
    version,
    about = "Trace and explain code execution",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a program under Xplainit tracing (live tracing entry point).
    Run {
        /// Path to the source file or program to trace.
        file: String,
    },

    /// Explain a function, given as `path::function` (e.g. `app.py::main`).
    Explain {
        /// Target in the form `file::function`.
        target: String,

        /// Optional trace JSON to explain recorded events for the function.
        #[arg(long)]
        trace: Option<String>,
    },

    /// Analyze a JSON event log and explain any error events.
    Analyze {
        /// Path to a JSON file containing an array of ExecutionEvent values.
        log: String,
    },

    /// Render a JSON event trace using the core formatters.
    Report {
        /// Path to a JSON file containing an array of ExecutionEvent values.
        trace: String,

        /// Output format: text, json, html, or markdown.
        #[arg(long, default_value = "text")]
        format: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => commands::run::handle(&file),
        Commands::Explain { target, trace } => commands::explain::handle(&target, trace.as_deref()),
        Commands::Analyze { log } => commands::analyze::handle(&log),
        Commands::Report { trace, format } => commands::report::handle(&trace, &format),
    }
}
