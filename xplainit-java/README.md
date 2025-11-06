# Xplainit Java

Natural language explanations for Java code execution.

## Requirements

- Java 11 or higher
- Maven 3.6+ (for building)
- Rust/Cargo (for building native library)

## Building

```bash
# Build with Maven (also builds Rust native library)
mvn clean package

# Or build Rust library separately
cargo build --release
```

This will generate:
- `target/xplainit-java-0.1.0.jar` (Java library)
- `target/release/libxplainit_java.so` (Linux native library)
- `target/release/libxplainit_java.dylib` (macOS native library)
- `target/release/xplainit_java.dll` (Windows native library)

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

```bash
# Compile example
javac -cp target/xplainit-java-0.1.0.jar examples/BasicExample.java

# Run example (Linux/macOS)
java -Djava.library.path=target/release -cp target/xplainit-java-0.1.0.jar:examples io.xplainit.examples.BasicExample

# Run example (Windows)
java -Djava.library.path=target\release -cp target\xplainit-java-0.1.0.jar;examples io.xplainit.examples.BasicExample
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
