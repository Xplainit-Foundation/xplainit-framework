//! Performance benchmarks for xplainit-core
//! 
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use xplainit_core::*;
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;
use std::f64::consts::PI;

fn create_test_event(event_type: &str) -> ExecutionEvent {
    let id = Uuid::new_v4();
    let timestamp = Utc::now();
    let location = SourceLocation {
        file: "bench.rs".to_string(),
        line: 10,
        column: 5,
        offset: 100,
    };
    
    match event_type {
        "function_enter" => ExecutionEvent::FunctionEnter {
            id,
            name: "test_function".to_string(),
            args: {
                let mut args = HashMap::new();
                args.insert("x".to_string(), Value::Integer(42));
                args.insert("y".to_string(), Value::String("test".to_string()));
                args
            },
            location,
            timestamp,
        },
        "variable_assign" => ExecutionEvent::VariableAssign {
            id,
            name: "result".to_string(),
            old_value: Some(Value::Float(0.0)),
            new_value: Value::Float(PI),
            location,
            timestamp,
        },
        "error" => ExecutionEvent::DivisionByZero {
            id,
            numerator: Value::Integer(100),
            denominator_var: Some("zero".to_string()),
            location,
            timestamp,
        },
        _ => panic!("Unknown event type"),
    }
}

fn bench_event_creation(c: &mut Criterion) {
    c.bench_function("event_creation", |b| {
        b.iter(|| {
            black_box(create_test_event("function_enter"));
        });
    });
}

fn bench_event_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_store");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("record", size), size, |b, &size| {
            let store = EventStore::with_capacity(size);
            let event = create_test_event("function_enter");
            
            b.iter(|| {
                store.record(black_box(event.clone()));
            });
        });
    }
    
    group.finish();
}

fn bench_filtering(c: &mut Criterion) {
    let mut group = c.benchmark_group("filtering");
    let config = Config::new(Language::Python);
    
    // AcceptAllFilter
    group.bench_function("accept_all", |b| {
        let filter = AcceptAllFilter;
        let event = create_test_event("function_enter");
        
        b.iter(|| {
            black_box(filter.should_capture(&event, &config));
        });
    });
    
    // FunctionFilter
    group.bench_function("function_filter", |b| {
        let mut filter = FunctionFilter::new();
        filter.exclude.insert("test_function".to_string());
        let event = create_test_event("function_enter");
        
        b.iter(|| {
            black_box(filter.should_capture(&event, &config));
        });
    });
    
    // ModuleFilter
    group.bench_function("module_filter", |b| {
        let filter = ModuleFilter::new();
        let event = create_test_event("function_enter");
        
        b.iter(|| {
            let location = event.location();
            black_box(filter.should_filter_file(&location.file));
        });
    });
    
    group.finish();
}

fn bench_explanation_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("explanation");
    let generator = ExplanationGenerator::new(VerbosityLevel::Normal);
    
    for event_type in ["function_enter", "variable_assign", "error"].iter() {
        group.bench_with_input(BenchmarkId::new("explain", event_type), event_type, |b, &event_type| {
            let event = create_test_event(event_type);
            
            b.iter(|| {
                black_box(generator.explain(&event));
            });
        });
    }
    
    group.finish();
}

fn bench_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("formatting");
    let events: Vec<_> = (0..100).map(|_| create_test_event("function_enter")).collect();
    
    // Text formatting
    group.bench_function("text_format", |b| {
        let formatter = TextFormatter::new(VerbosityLevel::Normal);
        
        b.iter(|| {
            black_box(formatter.format_events(&events));
        });
    });
    
    // JSON formatting
    group.bench_function("json_format", |b| {
        let formatter = JsonFormatter::new(true);
        
        b.iter(|| {
            black_box(formatter.format_events(&events));
        });
    });
    
    // HTML formatting
    group.bench_function("html_format", |b| {
        let formatter = HtmlFormatter::new();
        
        b.iter(|| {
            black_box(formatter.format_events(&events));
        });
    });
    
    group.finish();
}

fn bench_runtime_control(c: &mut Criterion) {
    let mut group = c.benchmark_group("control");
    let config = Config::new(Language::Python);
    
    // Enable/disable check (zero-overhead path)
    group.bench_function("is_enabled", |b| {
        let control = RuntimeControl::new(config.clone());
        
        b.iter(|| {
            black_box(control.is_enabled());
        });
    });
    
    // Event capture check with rate limiting
    group.bench_function("should_capture_event", |b| {
        let control = RuntimeControl::new(config.clone());
        
        b.iter(|| {
            black_box(control.should_capture_event());
        });
    });
    
    group.finish();
}

fn bench_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline");
    
    group.bench_function("full_pipeline", |b| {
        let filter = Box::new(AcceptAllFilter);
        let processors = ProcessorPipeline::new()
            .add_processor(Box::new(PassThroughProcessor));
        let sink = Box::new(MemorySink::new(10000));
        
        let mut pipeline = EventPipeline::new(filter, processors)
            .add_sink(sink);
        let event = create_test_event("function_enter");
        let config = Config::new(Language::Python);
        
        b.iter(|| {
            let _ = black_box(pipeline.handle_event(event.clone(), &config));
        });
    });
    
    group.finish();
}

fn bench_error_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_analysis");
    let explainer = ErrorExplainer::new();
    
    group.bench_function("analyze_error", |b| {
        let event = create_test_event("error");
        
        b.iter(|| {
            black_box(explainer.analyze(&event));
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_event_creation,
    bench_event_store,
    bench_filtering,
    bench_explanation_generation,
    bench_formatting,
    bench_runtime_control,
    bench_pipeline,
    bench_error_analysis,
);

criterion_main!(benches);
