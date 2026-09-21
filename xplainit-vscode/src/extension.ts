// Entry point: activate/deactivate and command/provider registration.

import * as vscode from "vscode";
import * as path from "path";
import { TraceProvider, describeEvent } from "./traceProvider";
import { ExplanationHoverProvider } from "./explanationHover";
import { ErrorPanel } from "./errorPanel";

let traceProvider: TraceProvider | undefined;
let errorPanel: ErrorPanel | undefined;
let output: vscode.OutputChannel | undefined;

export function activate(context: vscode.ExtensionContext): void {
  output = vscode.window.createOutputChannel("Xplainit");
  context.subscriptions.push(output);

  const traces = new TraceProvider(output);
  traceProvider = traces;
  context.subscriptions.push(traces);

  const panel = new ErrorPanel(traces);
  errorPanel = panel;
  context.subscriptions.push({ dispose: () => panel.dispose() });

  // Keep the error panel in sync when the loaded trace changes.
  context.subscriptions.push(
    traces.onDidChangeEvents(() => {
      void panel.refreshIfVisible();
    })
  );

  // Timeline tree view.
  context.subscriptions.push(
    vscode.window.registerTreeDataProvider("xplainitTimeline", traces)
  );

  // Hover explanations across all files (matched by trace location).
  const hover = new ExplanationHoverProvider(traces);
  context.subscriptions.push(
    vscode.languages.registerHoverProvider({ scheme: "file" }, hover)
  );

  // ===== Commands =====
  context.subscriptions.push(
    vscode.commands.registerCommand("xplainit.loadTrace", () =>
      loadTraceCommand(traces)
    ),
    vscode.commands.registerCommand("xplainit.recordTrace", () =>
      recordTraceCommand(traces)
    ),
    vscode.commands.registerCommand("xplainit.playbackTrace", () =>
      traces.playback()
    ),
    vscode.commands.registerCommand("xplainit.stepForward", () =>
      traces.stepForward()
    ),
    vscode.commands.registerCommand("xplainit.stepBackward", () =>
      traces.stepBackward()
    ),
    vscode.commands.registerCommand("xplainit.stopPlayback", () =>
      traces.stopPlayback()
    ),
    vscode.commands.registerCommand("xplainit.goToEvent", (index: number) =>
      traces.goToEvent(index)
    ),
    vscode.commands.registerCommand("xplainit.explainFunction", () =>
      explainFunctionCommand(traces)
    ),
    vscode.commands.registerCommand("xplainit.showErrorPanel", () =>
      panel.show()
    )
  );
}

export function deactivate(): void {
  errorPanel?.dispose();
  errorPanel = undefined;
  traceProvider = undefined;
  output = undefined;
}

async function loadTraceCommand(traces: TraceProvider): Promise<void> {
  const picked = await vscode.window.showOpenDialog({
    canSelectMany: false,
    openLabel: "Load Xplainit Trace",
    filters: { "Xplainit Trace": ["json", "xplainit-trace"], All: ["*"] },
  });
  if (!picked || picked.length === 0) {
    return;
  }
  try {
    await traces.loadFromFile(picked[0].fsPath);
    vscode.window.showInformationMessage(
      `Xplainit: loaded ${traces.getEvents().length} event(s).`
    );
  } catch (err) {
    vscode.window.showErrorMessage(
      `Xplainit: failed to load trace: ${errMessage(err)}`
    );
  }
}

async function recordTraceCommand(traces: TraceProvider): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  const defaultTarget = editor ? editor.document.fileName : "";
  const target = await vscode.window.showInputBox({
    prompt: "File or program to trace with the xplainit CLI",
    value: defaultTarget,
  });
  if (!target) {
    return;
  }

  const defaultOut = target ? `${stripExt(target)}.xplainit-trace` : "trace.xplainit-trace";
  const outUri = await vscode.window.showSaveDialog({
    saveLabel: "Save Recorded Trace",
    defaultUri: vscode.Uri.file(defaultOut),
    filters: { "Xplainit Trace": ["xplainit-trace", "json"] },
  });
  if (!outUri) {
    return;
  }

  try {
    await traces.recordTrace(target, outUri.fsPath);
  } catch (err) {
    vscode.window.showErrorMessage(
      `Xplainit: recording failed: ${errMessage(err)}`
    );
  }
}

async function explainFunctionCommand(traces: TraceProvider): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showInformationMessage("Xplainit: open a file first.");
    return;
  }
  const wordRange = editor.document.getWordRangeAtPosition(
    editor.selection.active
  );
  const name = wordRange ? editor.document.getText(wordRange) : undefined;
  if (!name) {
    vscode.window.showInformationMessage(
      "Xplainit: place the cursor on a function name."
    );
    return;
  }

  const events = traces.eventsForFunction(name);
  if (events.length === 0) {
    vscode.window.showInformationMessage(
      `Xplainit: no recorded events for "${name}". Load or record a trace first.`
    );
    return;
  }

  const md = new vscode.MarkdownString(undefined, true);
  md.appendMarkdown(`### Xplainit explanation for \`${name}\`\n\n`);
  md.appendMarkdown(`Recorded ${events.length} event(s):\n\n`);
  for (const event of events) {
    const loc = event.payload.location;
    const where = loc && loc.file ? ` (${path.basename(loc.file)}:${loc.line})` : "";
    md.appendMarkdown(`- ${describeEvent(event)}${where}\n`);
  }

  const doc = await vscode.workspace.openTextDocument({
    content: md.value,
    language: "markdown",
  });
  await vscode.window.showTextDocument(doc, { preview: true });
}

function stripExt(filePath: string): string {
  const ext = path.extname(filePath);
  return ext ? filePath.slice(0, -ext.length) : filePath;
}

function errMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}
