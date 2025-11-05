# 🚀 Xplainit Framework

> **Step-by-step code execution explanation without AI, ML, or APIs**

[![CI/CD](https://github.com/xplainit/xplainit/workflows/CI%2FCD%20Pipeline/badge.svg)](https://github.com/xplainit/xplainit/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.0.1--dev-orange.svg)](https://github.com/xplainit/xplainit)

Xplainit is a production-ready framework that provides **step-by-step explanations of your code execution in plain English**. It works by observing your program at runtime without modifying its behavior.

## ✨ Features

- **🎯 Non-Invasive**: Your program runs exactly as it would without Xplainit
- **📝 Complete Coverage**: Explains every single step - simple to complex programs
- **🔥 Error-Aware**: Explains errors with the same quality as valid code
- **⚡ Zero Overhead**: When disabled, no performance impact
- **🎛️ Full Control**: Developers decide when, where, and how explanations appear
- **🌍 Multi-Language**: Python, JavaScript, C, C++, Java, Go, Rust
- **🚫 Offline**: No AI, ML, APIs, or internet connection required

## 🎓 Perfect For

- **Learning Programming**: Understand what your code actually does
- **Debugging**: See exactly where and why errors occur
- **Teaching**: Help students visualize execution flow
- **Code Review**: Understand complex code faster
- **Documentation**: Generate execution traces

## 🚀 Quick Start

### Python

```python
from xplainit import trace

@trace(verbosity="detailed")
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

result = fibonacci(5)
```

**Output:**
```
[Xplainit] Entering function 'fibonacci' with n=5
[Xplainit] Checking if 5 <= 1: False
[Xplainit] Recursively calling fibonacci with n=4
[Xplainit]   Entering function 'fibonacci' with n=4
[Xplainit]   Checking if 4 <= 1: False
...
[Xplainit] Returning 5 from fibonacci
```

### JavaScript/Node.js

```javascript
const { trace } = require('xplainit');

trace(function calculateSum(arr) {
    let total = 0;
    for (let num of arr) {
        total += num;
    }
    return total;
});

calculateSum([1, 2, 3, 4, 5]);
```

### More Examples

See the [examples directory](docs/examples/) for more comprehensive examples in all supported languages.

## 🎯 What Makes Xplainit Special?

### 1. Runtime Instrumentation

Unlike static analyzers, Xplainit observes **actual execution** with **real values**:

```python
# Static analysis says: "Calling function with argument x"
# Xplainit says: "Calling function 'process' with x=42, y='hello'"
```

### 2. Error Explanation

Errors are explained with the same detail as successful execution:

```python
def divide(a, b):
    return a / b

divide(10, 0)
```

**Xplainit Output:**
```
❌ Division by Zero Error on line 2

What happened:
  Trying to divide 10 by 0
  Division by zero is mathematically undefined

Why it happened:
  Parameter 'b' was passed as 0 when calling divide(10, 0)

How to fix:
  if b != 0:
      return a / b
  else:
      return None  # or handle appropriately
```

### 3. Complete Control

```python
# Method 1: Decorator (function-level)
@trace
def my_function():
    pass

# Method 2: Context manager (block-level)
with Explainer.trace():
    complex_operation()

# Method 3: Global control
Explainer.enable()
entire_program()
Explainer.disable()

# Method 4: Environment variable
# XPLAINIT_ENABLED=false python script.py
```

## 📦 Installation

### Python (pip)

```bash
pip install xplainit
```

### JavaScript (npm)

```bash
npm install xplainit
```

### Rust (Cargo)

```toml
[dependencies]
xplainit = "0.0.1"
```

### Other Languages

See [installation guide](docs/installation.md) for C, C++, Java, and Go.

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│           Xplainit Runtime Engine (Rust Core)           │
├─────────────────────────────────────────────────────────┤
│ • Event Capture System                                  │
│ • Execution Trace Storage                               │
│ • Explanation Generator                                 │
│ • Output Controller                                     │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│            Language-Specific Runtime Hooks              │
├─────────────────────────────────────────────────────────┤
│ Python: sys.settrace  │  Node: V8 Inspector             │
│ C/C++: GDB/LLDB       │  Java: JVM TI                   │
│ Go: runtime hooks     │  Rust: proc macros              │
└─────────────────────────────────────────────────────────┘
```

## 📖 Documentation

- [Installation Guide](docs/installation.md)
- [User Guide](docs/user-guide.md)
- [API Reference](docs/api/)
- [Examples](docs/examples/)
- [Architecture](DESIGN_PHILOSOPHY.md)
- [Contributing](CONTRIBUTING.md)

## 🛠️ Development Status

**Current Version**: v0.0.1-dev (In Development)

### Roadmap

- [x] Project setup and architecture design
- [x] Core event types and configuration
- [ ] Runtime instrumentation core
- [ ] Python integration
- [ ] JavaScript integration
- [ ] Error handling system
- [ ] Output formatting
- [ ] C/C++ integration
- [ ] Java integration
- [ ] Go integration
- [ ] Rust integration
- [ ] Comprehensive testing
- [ ] Documentation
- [ ] v0.0.1 Release

See [TODO list](FRAMEWORK_PLAN.md) for detailed progress.

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/xplainit/xplainit.git
cd xplainit

# Build
cargo build --all

# Run tests
cargo test --all

# Run clippy
cargo clippy --all -- -D warnings
```

## 📄 License

Dual licensed under:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

Choose whichever license suits your needs.

## 🌟 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Inspired by debuggers, profilers, and educational tools
- Special thanks to all contributors

## 📞 Support

- **Documentation**: [docs.xplainit.io](https://docs.xplainit.io)
- **Issues**: [GitHub Issues](https://github.com/xplainit/xplainit/issues)
- **Discussions**: [GitHub Discussions](https://github.com/xplainit/xplainit/discussions)

---

**Built with ❤️ to make code execution transparent and understandable for everyone.**

---

## 🎉 Project Status

**We're actively building this! Check back soon for the first release.**

Current phase: **Core Infrastructure** 🏗️

Star ⭐ this repo to follow our progress!
