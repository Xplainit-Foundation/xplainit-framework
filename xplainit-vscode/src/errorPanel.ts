// Error analysis panel: a webview listing error events with CLI analysis text.

import * as vscode from "vscode";
import { TraceProvider, describeEvent } from "./traceProvider";
import { TraceEvent, eventType } from "./trace";

/// Manages a single reusable webview panel that shows the trace's error events
/// alongside the output of `xplainit analyze`.
export class ErrorPanel {
  private static readonly viewType = "xplainit.errorPanel";
  private panel: vscode.WebviewPanel | undefined;

  constructor(private readonly traces: TraceProvider) {}

  dispose(): void {
    this.panel?.dispose();
    this.panel = undefined;
  }

  /// Create (or reveal) the panel and refresh its contents.
  async show(): Promise<void> {
    if (!this.panel) {
      this.panel = vscode.window.createWebviewPanel(
        ErrorPanel.viewType,
        "Xplainit Error Analysis",
        vscode.ViewColumn.Beside,
        { enableScripts: false, retainContextWhenHidden: true }
      );
      this.panel.onDidDispose(() => {
        this.panel = undefined;
      });
    }
    this.panel.reveal(vscode.ViewColumn.Beside);
    await this.refresh();
  }

  /// Re-render if the panel is open (called when the trace changes).
  async refreshIfVisible(): Promise<void> {
    if (this.panel) {
      await this.refresh();
    }
  }

  private async refresh(): Promise<void> {
    if (!this.panel) {
      return;
    }
    const errors = this.traces.errorEvents();

    let analysis = "";
    let analysisError: string | undefined;
    if (this.traces.getTracePath()) {
      try {
        analysis = await this.traces.runAnalyze();
      } catch (err) {
        analysisError =
          err instanceof Error ? err.message : String(err);
      }
    } else {
      analysisError = "No trace file path available to run `xplainit analyze`.";
    }

    this.panel.webview.html = this.renderHtml(errors, analysis, analysisError);
  }

  private renderHtml(
    errors: TraceEvent[],
    analysis: string,
    analysisError: string | undefined
  ): string {
    const rows = errors
      .map((event) => {
        const loc = event.payload.location;
        const where = loc && loc.file ? `${escapeHtml(loc.file)}:${loc.line}` : "unknown";
        return `<tr>
          <td><code>${escapeHtml(eventType(event.variant))}</code></td>
          <td>${escapeHtml(describeEvent(event))}</td>
          <td>${where}</td>
        </tr>`;
      })
      .join("\n");

    const errorTable =
      errors.length > 0
        ? `<table>
            <thead><tr><th>Type</th><th>Description</th><th>Location</th></tr></thead>
            <tbody>${rows}</tbody>
          </table>`
        : `<p class="muted">No error events found in the loaded trace.</p>`;

    const analysisBlock = analysisError
      ? `<p class="muted">Could not run <code>xplainit analyze</code>: ${escapeHtml(analysisError)}</p>`
      : `<pre>${escapeHtml(stripAnsi(analysis))}</pre>`;

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Xplainit Error Analysis</title>
  <style>
    body { font-family: var(--vscode-font-family); padding: 12px; color: var(--vscode-foreground); }
    h1 { font-size: 1.2em; }
    h2 { font-size: 1.0em; margin-top: 1.5em; }
    table { border-collapse: collapse; width: 100%; }
    th, td { text-align: left; padding: 4px 8px; border-bottom: 1px solid var(--vscode-panel-border); }
    code { color: var(--vscode-textPreformat-foreground); }
    pre { white-space: pre-wrap; background: var(--vscode-textBlockQuote-background); padding: 8px; border-radius: 4px; }
    .muted { color: var(--vscode-descriptionForeground); }
  </style>
</head>
<body>
  <h1>Xplainit Error Analysis</h1>
  <p class="muted">${errors.length} error event(s) detected.</p>
  ${errorTable}
  <h2>CLI analysis (<code>xplainit analyze</code>)</h2>
  ${analysisBlock}
</body>
</html>`;
  }
}

/// Escape a string for safe insertion into HTML text/attribute content.
function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

/// Remove ANSI color escape codes that the colored CLI output may contain.
function stripAnsi(value: string): string {
  // eslint-disable-next-line no-control-regex
  return value.replace(/\u001b\[[0-9;]*m/g, "");
}
