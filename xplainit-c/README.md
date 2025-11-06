# Xplainit C/C++

Natural language explanations for C/C++ code execution.

## Building

```bash
cargo build --release
```

This will generate:
- `target/release/libxplainit_c.so` (Linux)
- `target/release/libxplainit_c.dylib` (macOS)
- `target/release/xplainit_c.dll` (Windows)
- `include/xplainit-c.h` (C header file)

## Installation

### Linux/macOS

```bash
# Copy library
sudo cp target/release/libxplainit_c.so /usr/local/lib/
# or on macOS:
# sudo cp target/release/libxplainit_c.dylib /usr/local/lib/

# Copy header
sudo cp include/xplainit-c.h /usr/local/include/

# Update library cache (Linux only)
sudo ldconfig
```

### Windows

Copy `xplainit_c.dll` and `xplainit-c.h` to your project directory.

## Usage

### C Example

```c
#include <stdio.h>
#include <xplainit-c.h>

int main(void) {
    // Create runtime
    XplainitHandle* handle = xplainit_create();
    
    // Enable tracing
    xplainit_enable(handle);
    
    // Your C code here...
    
    // Get statistics
    size_t total = 0, functions = 0, errors = 0;
    xplainit_get_statistics(handle, &total, &functions, &errors);
    printf("Captured %zu events\n", total);
    
    // Get events as JSON
    char* events = xplainit_get_events(handle);
    printf("Events: %s\n", events);
    xplainit_free_string(events);
    
    // Cleanup
    xplainit_disable(handle);
    xplainit_free(handle);
    
    return 0;
}
```

**Compile:**
```bash
gcc example.c -o example -lxplainit_c
```

### C++ Example with RAII Wrapper

```cpp
#include <iostream>
#include <xplainit-c.h>

class Xplainit {
    XplainitHandle* handle_;
public:
    Xplainit() : handle_(xplainit_create()) {}
    ~Xplainit() { xplainit_free(handle_); }
    
    void enable() { xplainit_enable(handle_); }
    void disable() { xplainit_disable(handle_); }
    
    std::string get_events() const {
        char* events = xplainit_get_events(handle_);
        std::string result(events);
        xplainit_free_string(events);
        return result;
    }
};

int main() {
    Xplainit tracer;
    tracer.enable();
    
    // Your C++ code here...
    
    std::cout << "Events: " << tracer.get_events() << std::endl;
    
    tracer.disable();
    return 0;
}
```

**Compile:**
```bash
g++ example.cpp -o example -lxplainit_c -std=c++11
```

## API Reference

### `XplainitHandle* xplainit_create()`

Create a new Xplainit runtime instance.

**Returns:** Pointer to handle on success, NULL on failure

**Note:** Must be freed with `xplainit_free()`

### `void xplainit_free(XplainitHandle* handle)`

Free a Xplainit runtime instance.

**Safety:** Handle must not be used after this call

### `int xplainit_enable(XplainitHandle* handle)`

Enable tracing for the runtime instance.

**Returns:** 1 on success, 0 on failure

### `int xplainit_disable(XplainitHandle* handle)`

Disable tracing for the runtime instance.

**Returns:** 1 on success, 0 on failure

### `int xplainit_is_enabled(XplainitHandle* handle)`

Check if tracing is currently enabled.

**Returns:** 1 if enabled, 0 if disabled

### `char* xplainit_get_events(XplainitHandle* handle)`

Get all captured events as a JSON string.

**Returns:** Pointer to null-terminated JSON string, NULL on failure

**Note:** Must be freed with `xplainit_free_string()`

### `int xplainit_clear_events(XplainitHandle* handle)`

Clear all captured events from memory.

**Returns:** 1 on success, 0 on failure

### `int xplainit_get_statistics(XplainitHandle* handle, size_t* total, size_t* functions, size_t* errors)`

Get statistics about captured events.

**Arguments:**
- `total` - Output for total event count (can be NULL)
- `functions` - Output for function call count (can be NULL)
- `errors` - Output for error count (can be NULL)

**Returns:** 1 on success, 0 on failure

### `void xplainit_free_string(char* s)`

Free a string returned by `xplainit_get_events()`.

**Safety:** String must not be used after this call

### `const char* xplainit_version()`

Get the Xplainit version string.

**Returns:** Pointer to static version string

## Performance

Xplainit is designed for minimal overhead:
- **<2μs per event** on modern hardware
- **1-2% overhead** for typical applications
- **Zero-cost** when disabled

## Thread Safety

All functions are thread-safe. Multiple threads can use the same handle concurrently.

## License

MIT OR Apache-2.0
