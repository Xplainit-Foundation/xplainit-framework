// Trace loading, indexing, step-through visualization, and record/playback.

import * as vscode from "vscode";
import * as cp from "child_process";
import * as fs from "fs";
import * as path from "path";
import {
  TraceEvent,
  EventVariant,
  parseTrace,
  eventType,
  isErrorVariant,
  formatValue,
} from "./trace";

/// Key used to index events by source location: "file:line".
function locationKey(file: string, line: number): string {
  return `${file}:${line}`;
}

/// Holds a loaded trace and the indexes derived from it. A single instance is
/// shared across the hover provider, timeline view, and error panel so they all
/// observe the same events and playback cursor.
export class TraceProvider implements vscode.TreeDataProvider<TimelineNode> {
  private events: TraceEvent[] = [];
  private tracePath: string | undefined;

  /// events indexed by "file:line" for fast hover lookup.
  private byLocation = new Map<string, TraceEvent[]>();
  /// events grouped by function name (FunctionEnter/Exit and nested events).
  private byFunction = new Map<string, TraceEvent[]>();

  /// Current playback cursor (index into `events`). -1 means not started.
  private cursor = -1;
  private highlightDecoration: vscode.TextEditorDecorationType;

  private readonly onDidChangeTreeDataEmitter = new vscode.EventEmitter<
    TimelineNode | undefined | void
  >();
  readonly onDidChangeTreeData = this.onDidChangeTreeDataEmitter.event;

  private readonly onDidChangeEventsEmitter = new vscode.EventEmitter<void>();
  /// Fires whenever the loaded trace changes (load, record, clear).
  readonly onDidChangeEvents = this.onDidChangeEventsEmitter.event;

  constructor(private readonly output: vscode.OutputChannel) {
    this.highlightDecoration = vscode.window.createTextEditorDecorationType({
      backgroundColor: new vscode.ThemeColor(
        "editor.findMatchHighlightBackground"
      ),
      isWholeLine: true,
      overviewRulerColor: new vscode.ThemeColor("editorOverviewRuler.infoForeground"),
      overviewRulerLane: vscode.OverviewRulerLane.Full,
    });
  }

  dispose(): void {
    this.highlightDecoration.dispose();
    this.onDidChangeTreeDataEmitter.dispose();
    this.onDidChangeEventsEmitter.dispose();
  }

  // ===== Loading and indexing =====

  getEvents(): readonly TraceEvent[] {
    return this.events;
  }

  getTracePath(): string | undefined {
    return this.tracePath;
  }

  /// Events recorded at the given file and line, in trace order.
  eventsAtLocation(file: string, line: number): TraceEvent[] {
    const exact = this.byLocation.get(locationKey(file, line));
    if (exact && exact.length > 0) {
      return exact;
    }
    // Fall back to matching by basename so that traces produced with a
    // different absolute prefix still line up with the open document.
    const base = path.basename(file);
    const matches: TraceEvent[] = [];
    for (const [key, evts] of this.byLocation) {
      const [keyFile, keyLine] = splitLocationKey(key);
      if (keyLine === line && path.basename(keyFile) === base) {
        matches.push(...evts);
      }
    }
    return matches;
  }

  async loadFromFile(filePath: string): Promise<void> {
    const bytes = await vscode.workspace.fs.readFile(vscode.Uri.file(filePath));
    const text = Buffer.from(bytes).toString("utf8");
    this.setEvents(parseTrace(text), filePath);
  }

  private setEvents(events: TraceEvent[], tracePath: string | undefined): void {
    this.events = events;
    this.tracePath = tracePath;
    this.cursor = -1;
    this.reindex();
    this.onDidChangeTreeDataEmitter.fire();
    this.onDidChangeEventsEmitter.fire();
  }

  private reindex(): void {
    this.byLocation.clear();
    this.byFunction.clear();
    let currentFunction: string | undefined;

    for (const event of this.events) {
      const loc = event.payload.location;
      if (loc && loc.file && loc.line > 0) {
        const key = locationKey(loc.file, loc.line);
        const bucket = this.byLocation.get(key);
        if (bucket) {
          bucket.push(event);
        } else {
          this.byLocation.set(key, [event]);
        }
      }

      if (event.variant === "FunctionEnter") {
        currentFunction = event.payload.name;
      }
      const fnName = event.payload.name ?? currentFunction;
      if (fnName) {
        const bucket = this.byFunction.get(fnName);
        if (bucket) {
          bucket.push(event);
        } else {
          this.byFunction.set(fnName, [event]);
        }
      }
      if (event.variant === "FunctionExit") {
        currentFunction = undefined;
      }
    }
  }

  eventsForFunction(name: string): TraceEvent[] {
    return this.byFunction.get(name) ?? [];
  }

  errorEvents(): TraceEvent[] {
    return this.events.filter((e) => isErrorVariant(e.variant));
  }

  // ===== Step-through playback =====

  async stepForward(): Promise<void> {
    if (this.events.length === 0) {
      vscode.window.showInformationMessage("Xplainit: no trace loaded.");
      return;
    }
    if (this.cursor < this.events.length - 1) {
      this.cursor += 1;
    }
    await this.revealCursor();
  }

  async stepBackward(): Promise<void> {
    if (this.events.length === 0) {
      return;
    }
    if (this.cursor > 0) {
      this.cursor -= 1;
    }
    await this.revealCursor();
  }

  async playback(): Promise<void> {
    if (this.events.length === 0) {
      vscode.window.showInformationMessage(
        "Xplainit: load or record a trace first."
      );
      return;
    }
    this.cursor = 0;
    await this.revealCursor();
    vscode.window.showInformationMessage(
      "Xplainit: playback started. Use Step Forward / Step Backward to move."
    );
  }

  stopPlayback(): void {
    this.cursor = -1;
    this.clearHighlights();
  }

  private currentEvent(): TraceEvent | undefined {
    if (this.cursor < 0 || this.cursor >= this.events.length) {
      return undefined;
    }
    return this.events[this.cursor];
  }

  private async revealCursor(): Promise<void> {
    const event = this.currentEvent();
    if (!event) {
      return;
    }
    this.onDidChangeTreeDataEmitter.fire();

    const loc = event.payload.location;
    if (!loc || !loc.file || loc.line <= 0) {
      vscode.window.setStatusBarMessage(
        `Xplainit: ${describeEvent(event)} (no source location)`,
        4000
      );
      return;
    }

    const uri = await this.resolveSourceUri(loc.file);
    if (!uri) {
      return;
    }
    const doc = await vscode.workspace.openTextDocument(uri);
    const editor = await vscode.window.showTextDocument(doc, {
      preserveFocus: false,
    });
    const lineIndex = Math.max(0, loc.line - 1);
    const range = doc.lineAt(Math.min(lineIndex, doc.lineCount - 1)).range;
    editor.selection = new vscode.Selection(range.start, range.start);
    editor.revealRange(range, vscode.TextEditorRevealType.InCenter);
    editor.setDecorations(this.highlightDecoration, [range]);
    vscode.window.setStatusBarMessage(
      `Xplainit [${this.cursor + 1}/${this.events.length}]: ${describeEvent(event)}`,
      6000
    );
  }

  private clearHighlights(): void {
    for (const editor of vscode.window.visibleTextEditors) {
      editor.setDecorations(this.highlightDecoration, []);
    }
  }

  /// Resolve a trace file path to a workspace document URI, matching by
  /// basename when the recorded path is not directly openable.
  private async resolveSourceUri(file: string): Promise<vscode.Uri | undefined> {
    if (path.isAbsolute(file)) {
      try {
        await vscode.workspace.fs.stat(vscode.Uri.file(file));
        return vscode.Uri.file(file);
      } catch {
        // Fall through to workspace search by basename.
      }
    }
    const base = path.basename(file);
    const found = await vscode.workspace.findFiles(
      `**/${base}`,
      "**/node_modules/**",
      1
    );
    if (found.length > 0) {
      return found[0];
    }
    vscode.window.showWarningMessage(
      `Xplainit: could not locate source file "${file}" in the workspace.`
    );
    return undefined;
  }

  // ===== Record via the xplainit CLI =====

  /// Record a trace by shelling out to the xplainit CLI (see FEAT-001).
  /// The CLI writes trace JSON to `outputPath`; we then load it.
  async recordTrace(targetFile: string, outputPath: string): Promise<void> {
    const config = vscode.workspace.getConfiguration("xplainit");
    const cliPath = config.get<string>("cliPath", "xplainit");
    const subcommand = config.get<string>("recordCommand", "run");

    this.output.appendLine(
      `[record] ${cliPath} ${subcommand} ${targetFile} > ${outputPath}`
    );

    await new Promise<void>((resolve, reject) => {
      const child = cp.spawn(cliPath, [subcommand, targetFile], {
        cwd: this.workspaceCwd(),
      });
      const outStream = fs.createWriteStream(outputPath);
      child.stdout.pipe(outStream);
      child.stderr.on("data", (chunk: Buffer) =>
        this.output.append(chunk.toString())
      );
      child.on("error", reject);
      child.on("close", (code) => {
        outStream.end();
        if (code === 0) {
          resolve();
        } else {
          reject(
            new Error(`xplainit ${subcommand} exited with code ${code ?? -1}.`)
          );
        }
      });
    });

    await this.loadFromFile(outputPath);
    vscode.window.showInformationMessage(
      `Xplainit: recorded trace to ${outputPath}.`
    );
  }

  /// Run `xplainit analyze <trace>` and return its stdout for the error panel.
  async runAnalyze(): Promise<string> {
    if (!this.tracePath) {
      throw new Error("No trace file is loaded to analyze.");
    }
    const config = vscode.workspace.getConfiguration("xplainit");
    const cliPath = config.get<string>("cliPath", "xplainit");
    return this.runCli(cliPath, ["analyze", this.tracePath]);
  }

  /// Run `xplainit report <trace> --format <fmt>` and return its stdout.
  async runReport(format: string): Promise<string> {
    if (!this.tracePath) {
      throw new Error("No trace file is loaded to report.");
    }
    const config = vscode.workspace.getConfiguration("xplainit");
    const cliPath = config.get<string>("cliPath", "xplainit");
    return this.runCli(cliPath, [
      "report",
      this.tracePath,
      "--format",
      format,
    ]);
  }

  private runCli(cliPath: string, args: string[]): Promise<string> {
    this.output.appendLine(`[cli] ${cliPath} ${args.join(" ")}`);
    return new Promise<string>((resolve, reject) => {
      const child = cp.spawn(cliPath, args, { cwd: this.workspaceCwd() });
      let stdout = "";
      let stderr = "";
      child.stdout.on("data", (chunk: Buffer) => (stdout += chunk.toString()));
      child.stderr.on("data", (chunk: Buffer) => (stderr += chunk.toString()));
      child.on("error", reject);
      child.on("close", (code) => {
        if (code === 0) {
          resolve(stdout);
        } else {
          reject(new Error(stderr || `CLI exited with code ${code ?? -1}.`));
        }
      });
    });
  }

  private workspaceCwd(): string | undefined {
    return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  }

  // ===== TreeDataProvider (timeline view) =====

  getTreeItem(element: TimelineNode): vscode.TreeItem {
    return element.toTreeItem(this.cursor);
  }

  getChildren(element?: TimelineNode): TimelineNode[] {
    if (element) {
      return [];
    }
    return this.events.map((event) => new TimelineNode(event));
  }

  /// Jump the playback cursor to a specific event (used by the timeline view).
  async goToEvent(index: number): Promise<void> {
    if (index < 0 || index >= this.events.length) {
      return;
    }
    this.cursor = index;
    await this.revealCursor();
  }
}

function splitLocationKey(key: string): [string, number] {
  const idx = key.lastIndexOf(":");
  const file = key.slice(0, idx);
  const line = Number(key.slice(idx + 1));
  return [file, line];
}

/// One row in the timeline tree.
export class TimelineNode {
  constructor(public readonly event: TraceEvent) {}

  toTreeItem(cursor: number): vscode.TreeItem {
    const item = new vscode.TreeItem(
      describeEvent(this.event),
      vscode.TreeItemCollapsibleState.None
    );
    item.description = eventType(this.event.variant);
    const loc = this.event.payload.location;
    if (loc && loc.file) {
      item.tooltip = `${loc.file}:${loc.line}`;
    }
    if (isErrorVariant(this.event.variant)) {
      item.iconPath = new vscode.ThemeIcon("error");
    } else if (this.event.index === cursor) {
      item.iconPath = new vscode.ThemeIcon("debug-stackframe-active");
    } else {
      item.iconPath = new vscode.ThemeIcon("circle-small");
    }
    item.command = {
      command: "xplainit.goToEvent",
      title: "Go To Event",
      arguments: [this.event.index],
    };
    return item;
  }
}

/// Short one-line description of an event for the timeline and status bar.
export function describeEvent(event: TraceEvent): string {
  const p = event.payload;
  switch (event.variant) {
    case "FunctionEnter":
      return `enter ${p.name ?? "?"}(${formatArgs(p.args)})`;
    case "FunctionExit":
      return `exit ${p.name ?? "?"} -> ${formatValue(p.return_value)}`;
    case "VariableDeclaration":
      return `let ${p.name ?? "?"} = ${formatValue(p.value)}`;
    case "VariableAssign":
      return `${p.name ?? "?"} = ${formatValue(p.new_value)}`;
    case "ConditionalEval":
      return `if (${p.condition ?? "?"}) -> ${p.result} [${p.branch_taken ?? ""}]`;
    case "LoopEntry":
      return `${p.loop_type ?? "loop"} enter`;
    case "LoopIteration":
      return `iteration ${p.iteration ?? "?"}`;
    case "LoopExit":
      return `loop exit (${p.total_iterations ?? 0} iterations, ${p.reason ?? ""})`;
    case "Return":
      return `return ${formatValue(p.value)}`;
    case "Exception":
      return `exception ${p.error_type ?? ""}: ${p.message ?? ""}`;
    case "SyntaxError":
      return `syntax error: ${p.message ?? ""}`;
    case "RuntimeError":
      return `runtime error ${p.error_type ?? ""}: ${p.message ?? ""}`;
    case "TypeError":
      return `type error: expected ${p.expected ?? "?"}, got ${p.got ?? "?"}`;
    case "NullPointerError":
      return `null access on ${p.variable ?? "?"}`;
    case "IndexOutOfBounds":
      return `index ${p.index ?? "?"} out of bounds (size ${p.size ?? "?"})`;
    case "DivisionByZero":
      return `division by zero`;
    case "StackOverflow":
      return `stack overflow in ${p.function ?? "?"} (depth ${p.recursion_depth ?? "?"})`;
    case "Panic":
      return `panic: ${p.message ?? ""}`;
    case "InfiniteLoopDetected":
      return `infinite loop (${p.iterations ?? "?"} iterations)`;
    case "DeadlockDetected":
      return `deadlock: ${(p.threads ?? []).join(", ")}`;
    case "MemoryLeakDetected":
      return `memory leak (${p.leaked_bytes ?? 0} bytes)`;
    default:
      return event.variant;
  }
}

function formatArgs(args: Record<string, unknown> | undefined): string {
  if (!args) {
    return "";
  }
  return Object.entries(args)
    .map(([name, value]) => `${name}=${formatValue(value)}`)
    .join(", ");
}
