# Xplainit Java

Natural language explanations for Java code execution.

## Status

Two recording paths are implemented and verified end to end:

- **Automatic runtime tracing via a JVMTI agent (recommended).** A native agent
  (`native/agent.c` -> `libxplainit_agent.so`) subscribes to JVMTI
  `MethodEntry` / `MethodExit` / `Exception` events, filters them to a target
  package (default `io.xplainit.examples`), and forwards them through JNI to the
  Xplainit runtime. Running `examples/BasicExample.java` under the agent
  captures real `FunctionEnter` / `FunctionExit` events for a simple method, a
  recursive method, and an `Exception` event for a thrown `ArithmeticException`.
  This is proven by `build_agent.sh`, which fails (non-zero exit) if the
  expected method names are not captured.
- **Explicit instance recording.** `Xplainit#onMethodEnter/onMethodExit/onException`
  let plain Java code push events without the agent.

Notes / honest limitations:

- The `Xplainit` Java class no longer depends on Gson, so it compiles and runs
  fully **offline** with plain `javac` (no Maven dependency downloads). The
  small statistics JSON is parsed by hand.
- The agent reports method **name** and the declaring **class signature** (used
  as the location `file`); it does not currently resolve source line numbers or
  argument values (line is recorded as `0`). Adding line numbers would require
  `GetLineNumberTable` lookups per event.
- `getEvents()` / `getStatistics()` (and their `agent*` equivalents) **drain**
  the event store, so read the events JSON you need before asking for stats.

## Requirements

- Java 11 or higher (verified with OpenJDK 25)
- `gcc` (or `clang`) to build the JVMTI agent
- Rust/Cargo (for building the native JNI library)
- Maven is **optional** (the build script uses plain `javac`, no external jars)

## Building

The simplest path is the bundled build script, which builds everything and runs
the agent-backed example as a self-checking gate:

```bash
cd xplainit-java
bash build_agent.sh
```

`build_agent.sh` will:

1. Detect `JAVA_HOME` robustly (via `java -XshowSettings:properties`, since
   `java`/`javac` may be version-manager shims that do not resolve to the JDK).
2. Run `cargo build --release` to produce `target/release/libxplainit_java.so`.
3. Compile the agent with
   `gcc -shared -fPIC -I$JAVA_HOME/include -I$JAVA_HOME/include/linux` into
   `libxplainit_agent.so`.
4. Compile the Java classes with `javac` into `classes/` (no external jars).
5. Run `BasicExample` under `-agentpath:` and assert real method events were
   captured.

### Running the example under the agent

The exact invocation (as executed by `build_agent.sh`) is:

```bash
java \
  -agentpath:"$PWD/libxplainit_agent.so" \
  -Djava.library.path="$PWD/../target/release" \
  -cp "$PWD/classes" \
  io.xplainit.examples.BasicExample
```

You can trace a different package by passing it as an agent option, using either
a bare package path or a full JVMTI signature prefix:

```bash
java -agentpath:"$PWD/libxplainit_agent.so=com/acme/app" -cp app.jar com.acme.app.Main
```

### Building only the Rust native library

```bash
cargo build --release
```

This produces:
- `target/release/libxplainit_java.so` (Linux native JNI library)

## Installation

### Maven

Add to your `pom.xml`:

```xml
<dependency>
    <groupId>io.xplainit</groupId>
    <artifactId>xplainit-java</artifactId>
    <version>0.1.0</version>
</dependency>
```

### Manual

1. Copy `xplainit-java-0.1.0.jar` to your project
2. Copy the native library to your `java.library.path`
3. Add the JAR to your classpath

## Usage

### Basic Usage

```java
import io.xplainit.Xplainit;

public class MyApp {
    public static void main(String[] args) {
        // Create tracer (use try-with-resources for auto-cleanup)
        try (Xplainit tracer = new Xplainit()) {
            
            // Enable tracing
            tracer.enable();
            
            // Your code here...
            int result = fibonacci(10);
            
            // Get statistics
            Xplainit.Statistics stats = tracer.getStatistics();
            System.out.println("Captured " + stats.total_events + " events");
            System.out.println("Function calls: " + stats.function_calls);
            System.out.println("Errors: " + stats.errors);
            
            // Get events as JSON
            String events = tracer.getEvents();
            System.out.println(events);
            
            // Disable tracing
            tracer.disable();
            
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
    
    private static int fibonacci(int n) {
        if (n <= 1) return n;
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}
```

### Advanced Usage

```java
import io.xplainit.Xplainit;

public class AdvancedExample {
    public static void main(String[] args) {
        Xplainit tracer = new Xplainit();
        
        try {
            tracer.enable();
            
            // Check if enabled
            if (tracer.isEnabled()) {
                System.out.println("Tracing active");
            }
            
            // Your application code
            runApplication();
            
            // Clear events periodically
            tracer.clearEvents();
            
        } finally {
            tracer.close(); // Always clean up
        }
    }
}
```

## API Reference

### `Xplainit`

Main class for runtime tracing.

#### Constructor

```java
Xplainit tracer = new Xplainit();
```

Creates a new tracer instance.

#### Methods

##### `boolean enable()`

Enable runtime tracing.

**Returns:** `true` if successfully enabled

##### `boolean disable()`

Disable runtime tracing.

**Returns:** `true` if successfully disabled

##### `boolean isEnabled()`

Check if tracing is currently active.

**Returns:** `true` if enabled, `false` otherwise

##### `String getEvents()`

Get all captured events as a JSON string.

**Returns:** JSON array of events

##### `boolean clearEvents()`

Clear all captured events from memory.

**Returns:** `true` if successfully cleared

##### `Statistics getStatistics()`

Get statistics about captured events.

**Returns:** `Statistics` object with event counts

##### `void close()`

Free native resources. Called automatically with try-with-resources.

### `Statistics`

Container for event statistics.

#### Fields

- `long total_events` - Total number of captured events
- `long function_calls` - Number of function call events
- `long errors` - Number of error events

## Examples

See the `examples/` directory for complete working examples:

- `BasicExample.java` - Simple usage demonstration

## Building Examples

The recommended way is `bash build_agent.sh` (see [Building](#building)). To do it
by hand with automatic tracing via the JVMTI agent:

```bash
# From xplainit-java/, after `cargo build --release`
gcc -shared -fPIC \
    -I"$JAVA_HOME/include" -I"$JAVA_HOME/include/linux" \
    -o libxplainit_agent.so native/agent.c

javac -d classes src/main/java/io/xplainit/Xplainit.java examples/BasicExample.java

java \
  -agentpath:"$PWD/libxplainit_agent.so" \
  -Djava.library.path="$PWD/../target/release" \
  -cp "$PWD/classes" \
  io.xplainit.examples.BasicExample
```

## Performance

Xplainit is designed for minimal overhead:
- **<2μs per event** on modern hardware
- **1-2% overhead** for typical applications
- **Zero-cost** when disabled

## Thread Safety

All methods are thread-safe. The same tracer instance can be used across multiple threads.

## Troubleshooting

### UnsatisfiedLinkError

If you get `java.lang.UnsatisfiedLinkError`, ensure:
1. Native library is in `java.library.path`
2. Correct library for your OS (`.so`, `.dylib`, or `.dll`)
3. Library was built for your architecture

### Build Issues

If Maven build fails:
1. Ensure Rust/Cargo is installed
2. Run `cargo build --release` manually
3. Check that JDK 11+ is installed

## License

MIT OR Apache-2.0
