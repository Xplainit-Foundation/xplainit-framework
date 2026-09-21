package xplainit

import (
	"encoding/json"
	"testing"
)

// event mirrors the externally-tagged JSON produced by the Rust core's
// serde serialization of ExecutionEvent, e.g. {"FunctionEnter": {...}}.
// Only the fields the tests assert on are declared.
type event struct {
	FunctionEnter *struct {
		Name string `json:"name"`
	} `json:"FunctionEnter"`
	FunctionExit *struct {
		Name string `json:"name"`
	} `json:"FunctionExit"`
	Exception *struct {
		ErrorType string `json:"error_type"`
		Message   string `json:"message"`
	} `json:"Exception"`
}

func newEnabledTracer(t *testing.T) *Xplainit {
	t.Helper()
	tracer := New()
	if tracer == nil {
		t.Fatal("New() returned nil; is the C library built and linkable?")
	}
	if !tracer.Enable() {
		t.Fatal("Enable() returned false")
	}
	if !tracer.IsEnabled() {
		t.Fatal("IsEnabled() returned false after Enable()")
	}
	return tracer
}

func TestVersion(t *testing.T) {
	if v := Version(); v == "" {
		t.Fatal("Version() returned an empty string")
	}
}

func TestRecordEnterExitStatistics(t *testing.T) {
	tracer := newEnabledTracer(t)
	defer tracer.Close()

	if !tracer.OnFunctionEnter("compute", "compute.go", 10) {
		t.Fatal("OnFunctionEnter returned false while enabled")
	}
	if !tracer.OnFunctionExit("compute", "compute.go", 10) {
		t.Fatal("OnFunctionExit returned false while enabled")
	}

	// GetStatistics drains the store, so read it before GetEvents.
	stats := tracer.GetStatistics()
	if stats.TotalEvents != 2 {
		t.Fatalf("expected 2 total events, got %d", stats.TotalEvents)
	}
	if stats.FunctionCalls != 2 {
		t.Fatalf("expected 2 function-call events, got %d", stats.FunctionCalls)
	}
	if stats.Errors != 0 {
		t.Fatalf("expected 0 errors, got %d", stats.Errors)
	}
}

func TestRecordEventsJSON(t *testing.T) {
	tracer := newEnabledTracer(t)
	defer tracer.Close()

	tracer.OnFunctionEnter("outer", "main.go", 1)
	tracer.OnFunctionEnter("inner", "main.go", 5)
	tracer.OnFunctionExit("inner", "main.go", 5)
	tracer.OnFunctionExit("outer", "main.go", 1)

	events := decodeEvents(t, tracer.GetEvents())
	if len(events) != 4 {
		t.Fatalf("expected 4 events, got %d", len(events))
	}

	var enters, exits []string
	for _, e := range events {
		switch {
		case e.FunctionEnter != nil:
			enters = append(enters, e.FunctionEnter.Name)
		case e.FunctionExit != nil:
			exits = append(exits, e.FunctionExit.Name)
		default:
			t.Fatalf("unexpected event variant: %+v", e)
		}
	}

	if !contains(enters, "outer") || !contains(enters, "inner") {
		t.Fatalf("expected enter events for outer and inner, got %v", enters)
	}
	if !contains(exits, "outer") || !contains(exits, "inner") {
		t.Fatalf("expected exit events for outer and inner, got %v", exits)
	}
}

func TestRecordException(t *testing.T) {
	tracer := newEnabledTracer(t)
	defer tracer.Close()

	if !tracer.OnException("DivisionByZero", "division by zero", "math.go", 42) {
		t.Fatal("OnException returned false while enabled")
	}

	events := decodeEvents(t, tracer.GetEvents())
	if len(events) != 1 {
		t.Fatalf("expected 1 event, got %d", len(events))
	}
	exc := events[0].Exception
	if exc == nil {
		t.Fatalf("expected an Exception event, got %+v", events[0])
	}
	if exc.ErrorType != "DivisionByZero" {
		t.Fatalf("expected error_type DivisionByZero, got %q", exc.ErrorType)
	}
	if exc.Message != "division by zero" {
		t.Fatalf("expected message %q, got %q", "division by zero", exc.Message)
	}
}

func TestExceptionReflectedInStatistics(t *testing.T) {
	tracer := newEnabledTracer(t)
	defer tracer.Close()

	tracer.OnFunctionEnter("f", "f.go", 1)
	tracer.OnException("BoomError", "boom", "f.go", 2)
	tracer.OnFunctionExit("f", "f.go", 1)

	// The enter + exit + exception are all captured in the total count.
	stats := tracer.GetStatistics()
	if stats.TotalEvents != 3 {
		t.Fatalf("expected 3 total events, got %d", stats.TotalEvents)
	}
	// Note: the C FFI statistics count only classifies the enum's dedicated
	// error variants (DivisionByZero/NullPointerError/IndexOutOfBounds) as
	// "errors". A generic Exception event (what OnException records) is
	// captured and appears in GetEvents but is not tallied into the Errors
	// statistic, so this stays 0. The exception itself is asserted via the
	// events JSON in TestRecordException.
	if stats.Errors != 0 {
		t.Fatalf("expected 0 counted errors for a generic Exception, got %d", stats.Errors)
	}
	if stats.FunctionCalls != 2 {
		t.Fatalf("expected 2 function-call events, got %d", stats.FunctionCalls)
	}
}

func TestTraceHelperRecordsEnterAndExit(t *testing.T) {
	tracer := newEnabledTracer(t)
	defer tracer.Close()

	func() {
		defer tracer.Trace("wrapped", "wrapped.go", 7)()
	}()

	events := decodeEvents(t, tracer.GetEvents())
	if len(events) != 2 {
		t.Fatalf("expected 2 events from Trace helper, got %d", len(events))
	}
	if events[0].FunctionEnter == nil || events[0].FunctionEnter.Name != "wrapped" {
		t.Fatalf("expected first event to be FunctionEnter for wrapped, got %+v", events[0])
	}
	if events[1].FunctionExit == nil || events[1].FunctionExit.Name != "wrapped" {
		t.Fatalf("expected second event to be FunctionExit for wrapped, got %+v", events[1])
	}
}

func TestRecordingIgnoredWhenDisabled(t *testing.T) {
	tracer := New()
	if tracer == nil {
		t.Fatal("New() returned nil")
	}
	defer tracer.Close()

	// A fresh tracer is enabled by default, so explicitly disable it first.
	if !tracer.Disable() {
		t.Fatal("Disable() returned false")
	}
	if tracer.IsEnabled() {
		t.Fatal("IsEnabled() returned true after Disable()")
	}

	// Tracing is disabled; recording should be a no-op.
	if tracer.OnFunctionEnter("x", "x.go", 1) {
		t.Fatal("OnFunctionEnter should return false when tracing is disabled")
	}

	stats := tracer.GetStatistics()
	if stats.TotalEvents != 0 {
		t.Fatalf("expected 0 events while disabled, got %d", stats.TotalEvents)
	}
}

func decodeEvents(t *testing.T, raw string) []event {
	t.Helper()
	var events []event
	if err := json.Unmarshal([]byte(raw), &events); err != nil {
		t.Fatalf("failed to parse events JSON %q: %v", raw, err)
	}
	return events
}

func contains(haystack []string, needle string) bool {
	for _, s := range haystack {
		if s == needle {
			return true
		}
	}
	return false
}
