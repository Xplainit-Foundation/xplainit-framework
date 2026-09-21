# Xplainit User Guide

Xplainit is a multi-language runtime code-execution explanation framework: a
Rust core engine (`xplainit-core`) with language binding layers (Python, Node,
C/C++, Java, Go), a CLI (`xplainit-cli`), and a web dashboard
(`xplainit-dashboard`). It captures execution events (function calls, variable
changes, control flow, errors, async task lifecycle) and turns them into
human-readable explanations.

This guide is plain Markdown. `mdbook` is **not available** in the offline build
environment, so this directory *is* the book: read it as linked Markdown here or
on the repository host. When `mdbook` and network access are available you can
render it as an HTML book (see below).

## Contents

### Introduction
- [Introduction](intro.md) — what Xplainit is, and an honest capability map.

### Getting started
- [Installation](getting-started/installation.md)
- [Quickstart](getting-started/quickstart.md)
- [Concepts](getting-started/concepts.md)

### Language guides
- [Python](guides/python.md)
- [JavaScript / Node](guides/javascript.md)
- [C / C++](guides/c-cpp.md)
- [Java](guides/java.md)
- [Go](guides/go.md)

### Advanced topics
- [Filtering](advanced/filtering.md)
- [Custom processors](advanced/custom-processors.md)
- [Performance tuning](advanced/performance-tuning.md)
- [Async tracing](advanced/async-tracing.md)

### Reference
The precise, source-grounded reference lives in [`../reference/`](../reference/):
[events](../reference/events.md), [configuration](../reference/configuration.md),
[filtering](../reference/filtering.md), [async](../reference/async.md). See also
[`../PERFORMANCE.md`](../PERFORMANCE.md), [`../SECURITY_AUDIT.md`](../SECURITY_AUDIT.md),
[`../TESTING.md`](../TESTING.md), and the project [`../STATUS.md`](../STATUS.md)
single source of truth.

## Building this guide as an HTML book (when online)

`mdbook` cannot be installed offline (`INTEGRATIONS_ONLY` sandbox). When network
access is available:

```sh
cargo install mdbook
# From the repo root, point mdbook at this directory:
mdbook build docs/guide -d ../../target/guide-book
# or preview live:
mdbook serve docs/guide
```

Until then, the Markdown files in this directory are the canonical guide.
