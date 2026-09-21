# Java

The Java binding (`xplainit-java`) provides JNI (Java Native Interface) bindings
over the core engine, with a Java `Xplainit` class and a Maven build.

## Build

```sh
cargo build --release -p xplainit-java
```

The Java side uses Maven (`pom.xml`, Java 11 target) and an `exec-maven-plugin`
step that builds the native Rust library; Gson is used for JSON parsing on the
Java side.

## API surface

The `Xplainit` Java class wraps native JNI methods (for example
`nativeCreate()`, `nativeFree()`, `nativeEnable()`, `nativeDisable()`,
`nativeIsEnabled()`, `nativeGetEvents()`, `nativeClearEvents()`,
`nativeGetStatistics()`), and implements `AutoCloseable` so it works with
try-with-resources. A `Statistics` inner class carries runtime metrics.

```java
try (Xplainit x = new Xplainit()) {
    x.enable();
    // ... run instrumented sections ...
    x.disable();
    // x.getEvents(), x.getStatistics()
}
```

## Safety

Every `unsafe` JNI block carries a `// SAFETY:` comment documenting its
invariants; raw-pointer entry points check for null before use (Phase 4,
Task 4.1). See [`../../SECURITY_AUDIT.md`](../../SECURITY_AUDIT.md).

## Scope

Like the other native bindings, this is a control/recording surface, not a full
automatic tracer for arbitrary JVM programs. Capture events through the API,
export them as an `ExecutionEvent` JSON array, then render/analyze with the CLI
or dashboard.
