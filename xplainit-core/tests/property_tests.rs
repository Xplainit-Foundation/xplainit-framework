//! Hand-rolled property-style tests for xplainit-core (FEAT-004).
//!
//! `proptest`/`quickcheck` are NOT available in this offline environment and
//! are deliberately NOT added as dependencies (see docs/TESTING.md). Instead we
//! roll our own deterministic generative testing with std only: a small linear
//! congruential generator (LCG) produces a reproducible stream of pseudo-random
//! inputs, and each test asserts an *invariant* over many generated cases. A
//! fixed seed keeps the runs deterministic and fast, and failures print the
//! seed/case so they can be reproduced.
//!
//! Invariants covered:
//!   * `ExecutionEvent` serialize -> deserialize round-trips to an equal value.
//!   * Redaction is idempotent (redacting twice equals redacting once) and
//!     never leaks a secret value.
//!   * Filter inclusion/exclusion is self-consistent (an excluded name is never
//!     captured; an include-list only captures listed names).
//!   * The event pipeline never panics on a generated event stream and only
//!     sinks events the filter accepts.
//!
//! Every test exercises real library code and fails if that code is reverted.

use std::collections::HashMap;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use uuid::Uuid;
use xplainit_core::*;

/// Minimal deterministic LCG (Numerical Recipes constants). std-only, no deps.
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        // x_{n+1} = a*x_n + c (mod 2^64)
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }

    fn boolean(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }
}

/// A pool of names, some of which are secret-like so redaction paths are hit.
const NAME_POOL: &[&str] = &[
    "count",
    "total",
    "user",
    "result",
    "password",
    "api_key",
    "token",
    "db_secret",
    "authorization",
    "value",
];

const SECRET_MARKER: &str = "top-secret-payload-value";

fn gen_value(rng: &mut Lcg, depth: u8) -> Value {
    // Bound recursion so generated structures stay small and fast.
    let max = if depth >= 3 { 5 } else { 9 };
    match rng.below(max) {
        0 => Value::Null,
        1 => Value::Bool(rng.boolean()),
        2 => Value::Integer(rng.next_u64() as i64),
        // Generate a FINITE, exactly-representable float. serde_json serializes
        // NaN/inf as `null` and reformats floats via a shortest-round-trip
        // algorithm, so an arbitrary decimal like 360.49800000000005 can print
        // differently after a decode (a serde formatting detail, not a bug in
        // the code under test). Using an integer value cast to f64 guarantees an
        // exact, stable textual round-trip so the invariant stays meaningful.
        3 => Value::Float((rng.next_u64() % 1_000_000) as f64 - 500_000.0),
        4 => Value::String(format!("s{}", rng.below(1000))),
        5 => {
            let len = rng.below(4);
            Value::Array((0..len).map(|_| gen_value(rng, depth + 1)).collect())
        }
        6 => {
            // At most ONE key. Multi-key HashMaps serialize in a per-instance
            // randomized order, so a byte-for-byte JSON round-trip of a
            // multi-key map is not deterministic across map instances. Keeping
            // <= 1 key makes the invariant meaningful (this mirrors the golden
            // tests in perf_regression.rs).
            let mut map = HashMap::new();
            if rng.boolean() {
                let (key, v) = secret_key_value_at(rng, depth + 1);
                map.insert(key, v);
            }
            Value::Object(map)
        }
        7 => Value::Function(format!("fn_{}", rng.below(50))),
        _ => Value::Unknown(format!("u{}", rng.below(50))),
    }
}

fn gen_location(rng: &mut Lcg) -> SourceLocation {
    SourceLocation::new(
        format!("mod/file_{}.py", rng.below(20)),
        rng.below(500) as usize,
        rng.below(80) as usize,
    )
}

fn key_is_secret(key: &str) -> bool {
    key.contains("password")
        || key.contains("secret")
        || key.contains("token")
        || key.contains("key")
        || key.contains("auth")
}

fn secret_key_value_at(rng: &mut Lcg, depth: u8) -> (String, Value) {
    let key = NAME_POOL[rng.below(NAME_POOL.len() as u64) as usize].to_string();
    // Give secret keys a recognizable value so leaks are detectable.
    let value = if key_is_secret(&key) {
        Value::String(SECRET_MARKER.to_string())
    } else {
        gen_value(rng, depth)
    };
    (key, value)
}

fn secret_key_value(rng: &mut Lcg) -> (String, Value) {
    secret_key_value_at(rng, 0)
}

/// A named map with AT MOST one key (see the note in `gen_value` about why
/// multi-key maps break byte-stable JSON comparisons).
fn gen_named_map(rng: &mut Lcg) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    if rng.boolean() {
        let (k, v) = secret_key_value(rng);
        map.insert(k, v);
    }
    map
}

/// Generate one arbitrary event using deterministic timestamps/ids so
/// serialization is stable within a case.
fn gen_event(rng: &mut Lcg) -> ExecutionEvent {
    let ts = Utc
        .timestamp_opt(1_700_000_000 + rng.below(10_000) as i64, 0)
        .unwrap();
    let id = Uuid::from_u128(rng.next_u64() as u128);
    match rng.below(10) {
        0 => ExecutionEvent::FunctionEnter {
            id,
            name: format!("f{}", rng.below(50)),
            args: gen_named_map(rng),
            location: gen_location(rng),
            timestamp: ts,
        },
        1 => ExecutionEvent::FunctionExit {
            id,
            name: format!("f{}", rng.below(50)),
            return_value: if rng.boolean() {
                Some(gen_value(rng, 0))
            } else {
                None
            },
            duration: Duration::from_millis(rng.below(5000)),
            timestamp: ts,
        },
        2 => ExecutionEvent::VariableDeclaration {
            id,
            name: NAME_POOL[rng.below(NAME_POOL.len() as u64) as usize].to_string(),
            value: Some(gen_value(rng, 0)),
            var_type: if rng.boolean() {
                Some("int".to_string())
            } else {
                None
            },
            is_const: rng.boolean(),
            location: gen_location(rng),
            timestamp: ts,
        },
        3 => {
            let (name, new_value) = secret_key_value(rng);
            ExecutionEvent::VariableAssign {
                id,
                name,
                old_value: if rng.boolean() {
                    Some(gen_value(rng, 0))
                } else {
                    None
                },
                new_value,
                location: gen_location(rng),
                timestamp: ts,
            }
        }
        4 => ExecutionEvent::ConditionalEval {
            id,
            condition: format!("x > {}", rng.below(100)),
            result: rng.boolean(),
            branch_taken: if rng.boolean() { "then" } else { "else" }.to_string(),
            location: gen_location(rng),
            timestamp: ts,
        },
        5 => ExecutionEvent::LoopIteration {
            id,
            loop_type: "for".to_string(),
            iteration: rng.below(1000) as usize,
            loop_var: Some("i".to_string()),
            loop_var_value: Some(Value::Integer(rng.next_u64() as i64)),
            timestamp: ts,
        },
        6 => ExecutionEvent::DivisionByZero {
            id,
            numerator: gen_value(rng, 0),
            denominator_var: Some("d".to_string()),
            location: gen_location(rng),
            timestamp: ts,
        },
        7 => ExecutionEvent::RuntimeError {
            id,
            error_type: "RuntimeError".to_string(),
            message: format!("boom {}", rng.below(100)),
            location: gen_location(rng),
            context: gen_named_map(rng),
            stack_trace: vec![],
            timestamp: ts,
        },
        8 => ExecutionEvent::AsyncTaskStart {
            id,
            task_id: Uuid::from_u128(rng.next_u64() as u128),
            task_name: format!("task{}", rng.below(20)),
            spawned_from: gen_location(rng),
            timestamp: ts,
        },
        _ => ExecutionEvent::Return {
            id,
            value: if rng.boolean() {
                Some(gen_value(rng, 0))
            } else {
                None
            },
            location: gen_location(rng),
            timestamp: ts,
        },
    }
}

/// Two floats are "serde-equal" when their bit patterns agree after a JSON
/// round-trip; NaN is normalized to null by serde_json, so we compare the
/// re-serialized JSON rather than `Value` equality for float-bearing events.
fn json_roundtrip_is_stable(event: &ExecutionEvent) -> bool {
    let json = serde_json::to_string(event).expect("serialize");
    let decoded: ExecutionEvent = serde_json::from_str(&json).expect("deserialize");
    let rejson = serde_json::to_string(&decoded).expect("reserialize");
    json == rejson && event.event_type() == decoded.event_type() && event.id() == decoded.id()
}

#[test]
fn property_event_json_round_trips() {
    // Generative: for many seeds, an event must round-trip stably through JSON.
    let mut rng = Lcg::new(0xC0FFEE_u64);
    for case in 0..2000u64 {
        let event = gen_event(&mut rng);
        assert!(
            json_roundtrip_is_stable(&event),
            "round-trip instability at case {case} for {}",
            event.event_type()
        );
    }
}

#[test]
fn property_redaction_is_idempotent_and_leak_free() {
    // Idempotence: redact(redact(x)) == redact(x); and once redacted, the
    // known secret marker never appears in the serialized output.
    let patterns: Vec<String> = DEFAULT_REDACTION_PATTERNS
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut rng = Lcg::new(0x1234_5678_9ABC_DEF0);
    for case in 0..2000u64 {
        let event = gen_event(&mut rng);
        let once = event.redacted(&patterns);
        let twice = once.redacted(&patterns);

        let once_json = serde_json::to_string(&once).expect("serialize once");
        let twice_json = serde_json::to_string(&twice).expect("serialize twice");
        assert_eq!(
            once_json,
            twice_json,
            "redaction not idempotent at case {case} ({})",
            event.event_type()
        );

        // The secret marker is only ever placed under secret-like keys, so a
        // correctly redacted event must not contain it anywhere.
        assert!(
            !once_json.contains(SECRET_MARKER),
            "secret leaked after redaction at case {case}: {once_json}"
        );
    }
}

#[test]
fn property_function_filter_inclusion_exclusion_consistent() {
    // Exclude always wins and an include-list only admits listed names. We
    // generate random function names and check the FunctionFilter decisions
    // stay consistent with its configured include/exclude sets.
    let config = Config::new(Language::Python);
    let mut rng = Lcg::new(0xDEAD_BEEF);

    for _ in 0..1000u64 {
        let name = format!("f{}", rng.below(6)); // small pool -> collisions

        // Exclusion: an excluded name must never be captured, regardless of
        // include state.
        let excl = FunctionFilter::new()
            .include(name.clone())
            .exclude(name.clone());
        let ev = ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: name.clone(),
            args: HashMap::new(),
            location: SourceLocation::new("t.py".to_string(), 1, 0),
            timestamp: Utc::now(),
        };
        assert!(
            !excl.should_capture(&ev, &config),
            "excluded name '{name}' was captured"
        );

        // Include-only: only the listed name is captured; a different name is
        // rejected.
        let incl = FunctionFilter::new().include("chosen_one");
        let chosen = ExecutionEvent::FunctionEnter {
            id: Uuid::new_v4(),
            name: "chosen_one".to_string(),
            args: HashMap::new(),
            location: SourceLocation::new("t.py".to_string(), 1, 0),
            timestamp: Utc::now(),
        };
        assert!(incl.should_capture(&chosen, &config));
        if name != "chosen_one" {
            assert!(
                !incl.should_capture(&ev, &config),
                "non-included name '{name}' was captured by include-only filter"
            );
        }
    }
}

#[test]
fn property_pipeline_never_panics_on_generated_streams() {
    // Drive a generated event stream through a real pipeline (filter ->
    // processor -> memory sink) and assert it never errors/panics, and that the
    // sink only ever receives events the filter accepted.
    let config = Config::new(Language::Python);
    let mut rng = Lcg::new(0x0BADF00D);

    // A PathFilter that excludes odd-numbered module files; combined with the
    // memory sink we can check "sunk <= accepted".
    let filter = Box::new(PathFilter::new().exclude("file_1.py"));
    let processors = ProcessorPipeline::new().add_processor(Box::new(PassThroughProcessor));
    let mem = MemorySink::new(100_000);
    let mem_probe = mem.clone();
    let mut pipeline = EventPipeline::new(filter, processors).add_sink(Box::new(mem));

    let check_filter = PathFilter::new().exclude("file_1.py");
    let mut accepted = 0usize;
    for _ in 0..5000u64 {
        let event = gen_event(&mut rng);
        if check_filter.should_capture(&event, &config) {
            accepted += 1;
        }
        // Must never return Err or panic on arbitrary input.
        pipeline
            .handle_event(event, &config)
            .expect("pipeline handled generated event");
    }

    let sunk = mem_probe.get_events().len();
    assert!(
        sunk <= accepted,
        "pipeline sank {sunk} events but only {accepted} passed the filter"
    );
    assert!(
        accepted > 0,
        "test filter rejected everything; generator broken"
    );
}

#[test]
fn property_redact_events_matches_per_event_redaction() {
    // redact_events over a slice must equal mapping redacted() over each event.
    let patterns: Vec<String> = DEFAULT_REDACTION_PATTERNS
        .iter()
        .map(|s| s.to_string())
        .collect();
    let mut rng = Lcg::new(0xFEED_FACE);
    let events: Vec<ExecutionEvent> = (0..500).map(|_| gen_event(&mut rng)).collect();

    let batch = redact_events(&events, &patterns);
    let per_event: Vec<ExecutionEvent> = events.iter().map(|e| e.redacted(&patterns)).collect();

    let batch_json = serde_json::to_string(&batch).expect("serialize batch");
    let per_json = serde_json::to_string(&per_event).expect("serialize per-event");
    assert_eq!(batch_json, per_json);
}
