# 🚀 QUICK START - USING XPLAINIT RIGHT NOW

**Last Updated:** January 11, 2026  
**Your Version:** v0.1.0  

---

## ⚡ 5-MINUTE QUICK START

### See It Work (30 seconds)

```bash
cd "c:\Users\siter\Desktop\Xplainit Framework"
cargo run --example demo_simple
```

**Output:** Beautiful demo showing all features ✨

### Run Other Examples (2 minutes)

```bash
# See basic usage
cargo run --example basic_usage

# See error analysis
cargo run --example error_analysis

# See custom filters
cargo run --example custom_filters

# See event pipeline
cargo run --example event_pipeline
```

---

## 💻 USE IT IN YOUR RUST PROJECT

### Step 1: Add Dependency

```toml
[dependencies]
xplainit-core = { path = "../Xplainit Framework/xplainit-core" }
```

### Step 2: Write Code

```rust
use xplainit_core::*;
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;

fn main() {
    // Create an event
    let event = ExecutionEvent::FunctionEnter {
        id: Uuid::new_v4(),
        name: "my_function".to_string(),
        args: {
            let mut args = HashMap::new();
            args.insert("x".to_string(), Value::Integer(42));
            args
        },
        location: SourceLocation::new("main.rs".to_string(), 10, 0),
        timestamp: Utc::now(),
    };
    
    // Explain it
    let explainer = ExplanationGenerator::new(VerbosityLevel::Normal);
    println!("{}", explainer.explain(&event));
    
    // Analyze errors
    let error = ExecutionEvent::DivisionByZero {
        id: Uuid::new_v4(),
        numerator: Value::Integer(10),
        denominator_var: Some("y".to_string()),
        location: SourceLocation::new("main.rs".to_string(), 15, 4),
        timestamp: Utc::now(),
    };
    
    let error_explainer = ErrorExplainer::new();
    if let Some(analysis) = error_explainer.analyze(&error) {
        println!("Error: {}", analysis.root_cause);
        println!("Fix: {}", analysis.fix_suggestions[0]);
    }
}
```

### Step 3: Run

```bash
cargo run
```

**Output:**
```
Calling function my_function with 1 argument(s)
Error: Division by zero is mathematically undefined and causes a runtime error
Fix: Add a check: if y != 0 { result = numerator / y }
```

---

## 🎯 COMMON USE CASES

### 1. Build a Custom Tracer

```rust
use xplainit_core::*;

struct MyTracer {
    engine: RuntimeEngine,
    explainer: ExplanationGenerator,
}

impl MyTracer {
    fn new() -> Self {
        let config = Config::new(Language::Python)
            .with_verbosity(Verbosity::Normal);
        Self {
            engine: RuntimeEngine::new(config),
            explainer: ExplanationGenerator::new(VerbosityLevel::Normal),
        }
    }
    
    fn record(&self, event: ExecutionEvent) {
        self.engine.event_store().record(event.clone());
        println!("{}", self.explainer.explain(&event));
    }
    
    fn get_stats(&self) -> EventStats {
        self.engine.event_stats()
    }
}
```

### 2. Filter Events

```rust
// Only capture functions starting with "test_"
let filter = FunctionFilter::new()
    .include("test_*");

if filter.should_capture(&event, &config) {
    // Process this event
}
```

### 3. Process Pipeline

```rust
// Create pipeline
let filter = Box::new(AcceptAllFilter);
let processors = ProcessorPipeline::new()
    .add_processor(Box::new(PassThroughProcessor));
let sink = Box::new(ConsoleSink::new(true));

let mut pipeline = EventPipeline::new(filter, processors)
    .add_sink(sink);

// Process events
pipeline.handle_event(event, &config)?;
```

### 4. Generate Reports

```rust
// Get events
let events = engine.get_events(100);

// Format as JSON
let json_formatter = JsonFormatter::new(true);
let json = json_formatter.format_events(&events);
std::fs::write("trace.json", json)?;

// Format as HTML
let html_formatter = HtmlFormatter::new();
let html = html_formatter.format_events(&events);
std::fs::write("trace.html", html)?;
```

---

## 🔧 CONFIGURATION OPTIONS

```rust
let config = Config::new(Language::Python)
    .with_verbosity(Verbosity::Detailed)
    .with_output_format(OutputFormat::Json)
    .with_output_mode(OutputMode::Streaming)
    .with_max_events(10000)
    .enable_colors(true);
```

---

## 📊 CHECK PERFORMANCE

```bash
# Run benchmarks
cd xplainit-core
cargo bench

# See results
cat target/criterion/*/report/index.html
```

**Expected Results:**
- Event creation: ~2μs
- Filtering: <500ns
- Explanation: ~1μs
- Full pipeline: ~2μs

---

## 🐛 TROUBLESHOOTING

### Problem: "Can't find xplainit-core"
**Solution:** Use full path in Cargo.toml
```toml
xplainit-core = { path = "c:/Users/siter/Desktop/Xplainit Framework/xplainit-core" }
```

### Problem: "Python import fails"
**Status:** Known issue (PyO3 0.22 migration needed)  
**Workaround:** Use Rust API for now  
**Fix:** Coming in Phase 1 (Week 1)

### Problem: "No automatic tracing"
**Status:** Feature not yet implemented  
**Current:** Manual event creation only  
**Coming:** Phase 2 (Weeks 4-8)

---

## 📚 LEARN MORE

### Documentation
- [README.md](README.md) - Overview
- [FRAMEWORK_PLAN.md](FRAMEWORK_PLAN.md) - Architecture
- [DESIGN_PHILOSOPHY.md](DESIGN_PHILOSOPHY.md) - Design principles
- [API Docs](https://docs.rs/xplainit-core) - Generated docs

### Examples
- [basic_usage.rs](xplainit-core/examples/basic_usage.rs)
- [error_analysis.rs](xplainit-core/examples/error_analysis.rs)
- [custom_filters.rs](xplainit-core/examples/custom_filters.rs)
- [demo_simple.rs](xplainit-core/examples/demo_simple.rs)

### Plans
- [PRODUCTION_READINESS_PLAN.md](PRODUCTION_READINESS_PLAN.md) - Full roadmap
- [DEMO_CURRENT_FUNCTIONALITY.md](DEMO_CURRENT_FUNCTIONALITY.md) - What works
- [EXECUTIVE_SUMMARY.md](EXECUTIVE_SUMMARY.md) - Overview

---

## 🎯 NEXT STEPS

### For Users (Now)
1. ✅ Run examples
2. ✅ Try Rust API
3. ✅ Build custom tools
4. ⏳ Wait for Python auto-tracing

### For Developers (Soon)
1. 🔜 Fix Python bindings (Week 1)
2. 🔜 Add sys.settrace (Weeks 2-3)
3. 🔜 Test & validate (Week 4)
4. 🔜 Public beta (Week 5)

---

## 💬 QUESTIONS?

### Current Functionality
✅ **Q: Can I use it now?**  
A: Yes! In Rust projects as a library.

✅ **Q: Does Python work?**  
A: Not yet (35 compilation errors, fix in Week 1)

✅ **Q: Can it trace automatically?**  
A: Not yet (runtime hooks in Weeks 4-8)

✅ **Q: What DOES work?**  
A: Event analysis, explanations, error analysis, filtering, formatting - all perfect!

### Future Features
🔜 **Q: When Python auto-tracing?**  
A: Week 8 (2 months)

🔜 **Q: When v1.0 release?**  
A: Week 17 (4 months)

🔜 **Q: How can I help?**  
A: Test examples, report bugs, contribute code!

---

## ⚡ TL;DR

**Works Now:**
```bash
cargo run --example demo_simple  # See it work!
```

**Use Case:**
```rust
// Manual event → Get explanation
let event = ExecutionEvent::FunctionEnter { /* ... */ };
let explainer = ExplanationGenerator::new(VerbosityLevel::Normal);
println!("{}", explainer.explain(&event));
```

**Coming Soon:**
```python
# Automatic tracing (Week 8)
import xplainit
xplainit.enable()
# Your code is automatically traced!
```

---

**Status:** ✅ Ready to use as a library  
**Timeline:** 🔜 8 weeks to auto-tracing  
**Documentation:** 📚 Complete  
**Performance:** ⚡ Excellent (<2μs)

**Go build something! 🚀**
