// Hover provider: shows what Xplainit recorded at the hovered source line.

import * as vscode from "vscode";
import { TraceProvider, describeEvent } from "./traceProvider";
import { TraceEvent, isErrorVariant, eventType, formatValue } from "./trace";

/// Provides inline hover explanations by matching trace events to the file and
/// line under the cursor. Backed by the shared TraceProvider so it always sees
/// the currently loaded trace.
export class ExplanationHoverProvider implements vscode.HoverProvider {
  constructor(private readonly traces: TraceProvider) {}

  provideHover(
    document: vscode.TextDocument,
    position: vscode.Position
  ): vscode.ProviderResult<vscode.Hover> {
    // Editor lines are zero-based; trace lines are one-based.
    const line = position.line + 1;
    const events = this.traces.eventsAtLocation(document.fileName, line);
    if (events.length === 0) {
      return undefined;
    }

    const markdown = new vscode.MarkdownString(undefined, true);
    markdown.isTrusted = false;
    markdown.appendMarkdown(
      `**Xplainit** recorded ${events.length} event(s) on this line:\n\n`
    );

    for (const event of events) {
      markdown.appendMarkdown(renderEvent(event));
      markdown.appendMarkdown("\n");
    }

    return new vscode.Hover(markdown);
  }
}

/// Render a single event as a Markdown list item with variant-specific detail.
function renderEvent(event: TraceEvent): string {
  const icon = isErrorVariant(event.variant) ? "$(error)" : "$(debug-stackframe-dot)";
  const header = `${icon} \`${eventType(event.variant)}\` - ${describeEvent(event)}`;
  const lines: string[] = [`- ${header}`];

  const p = event.payload;
  switch (event.variant) {
    case "FunctionEnter":
      if (p.args && Object.keys(p.args).length > 0) {
        for (const [name, value] of Object.entries(p.args)) {
          lines.push(`  - arg \`${name}\` = \`${formatValue(value)}\``);
        }
      }
      break;
    case "FunctionExit":
      lines.push(`  - returns \`${formatValue(p.return_value)}\``);
      break;
    case "VariableAssign":
      lines.push(
        `  - \`${p.name}\`: \`${formatValue(p.old_value)}\` -> \`${formatValue(p.new_value)}\``
      );
      break;
    case "VariableDeclaration":
      lines.push(
        `  - declared \`${p.name}\`${p.var_type ? ` : ${p.var_type}` : ""} = \`${formatValue(p.value)}\``
      );
      break;
    case "ConditionalEval":
      lines.push(
        `  - condition \`${p.condition}\` evaluated to **${p.result}** (branch: ${p.branch_taken})`
      );
      break;
    case "Exception":
    case "RuntimeError":
      lines.push(`  - ${p.error_type ?? "error"}: ${p.message ?? ""}`);
      break;
    case "TypeError":
      lines.push(
        `  - expected \`${p.expected}\`, got \`${p.got}\` during \`${p.operation}\``
      );
      break;
    case "IndexOutOfBounds":
      lines.push(
        `  - index \`${p.index}\` into \`${p.collection}\` of size \`${p.size}\``
      );
      break;
    case "DivisionByZero":
      lines.push(
        `  - numerator \`${formatValue(p.numerator)}\`, denominator \`${p.denominator_var ?? "?"}\` was zero`
      );
      break;
    default:
      break;
  }

  if (p.timestamp) {
    lines.push(`  - at \`${p.timestamp}\``);
  }
  return lines.join("\n");
}
