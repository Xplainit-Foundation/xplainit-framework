package xplainit

/*
#cgo CFLAGS: -I${SRCDIR}/../xplainit-c/include
#cgo linux LDFLAGS: -L${SRCDIR}/../target/release -lxplainit_c
#cgo darwin LDFLAGS: -L${SRCDIR}/../target/release -lxplainit_c
#cgo windows LDFLAGS: -L${SRCDIR}/../target/release -lxplainit_c.dll

#include <xplainit-c.h>
#include <stdlib.h>
*/
import "C"
import (
	"unsafe"
)

// Xplainit is the main tracer struct
type Xplainit struct {
	handle *C.XplainitHandle
}

// Statistics contains event statistics
type Statistics struct {
	TotalEvents   int `json:"total_events"`
	FunctionCalls int `json:"function_calls"`
	Errors        int `json:"errors"`
}

// New creates a new Xplainit tracer instance
func New() *Xplainit {
	handle := C.xplainit_create()
	if handle == nil {
		return nil
	}
	return &Xplainit{handle: handle}
}

// Close frees the native resources
func (x *Xplainit) Close() error {
	if x.handle != nil {
		C.xplainit_free(x.handle)
		x.handle = nil
	}
	return nil
}

// Enable turns on tracing
func (x *Xplainit) Enable() bool {
	if x.handle == nil {
		return false
	}
	result := C.xplainit_enable(x.handle)
	return result != 0
}

// Disable turns off tracing
func (x *Xplainit) Disable() bool {
	if x.handle == nil {
		return false
	}
	result := C.xplainit_disable(x.handle)
	return result != 0
}

// IsEnabled checks if tracing is active
func (x *Xplainit) IsEnabled() bool {
	if x.handle == nil {
		return false
	}
	result := C.xplainit_is_enabled(x.handle)
	return result != 0
}

// GetEvents returns all captured events as JSON string
func (x *Xplainit) GetEvents() string {
	if x.handle == nil {
		return "[]"
	}

	cStr := C.xplainit_get_events(x.handle)
	if cStr == nil {
		return "[]"
	}
	defer C.xplainit_free_string(cStr)

	return C.GoString(cStr)
}

// ClearEvents removes all captured events
func (x *Xplainit) ClearEvents() bool {
	if x.handle == nil {
		return false
	}
	result := C.xplainit_clear_events(x.handle)
	return result != 0
}

// GetStatistics returns statistics about captured events
func (x *Xplainit) GetStatistics() *Statistics {
	if x.handle == nil {
		return &Statistics{}
	}

	var total, functions, errors C.size_t
	C.xplainit_get_statistics(x.handle, &total, &functions, &errors)

	return &Statistics{
		TotalEvents:   int(total),
		FunctionCalls: int(functions),
		Errors:        int(errors),
	}
}

// OnFunctionEnter records a function entry event through the C FFI.
//
// The event is only recorded when tracing is enabled. name and file are
// passed to the native library as null-terminated C strings; line is the
// source line number. Returns true if the event was recorded.
//
// Go has no runtime equivalent of Python's sys.settrace, so these events
// are recorded manually by the caller (see Trace for the idiomatic helper).
func (x *Xplainit) OnFunctionEnter(name, file string, line int) bool {
	if x.handle == nil {
		return false
	}

	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))
	cFile := C.CString(file)
	defer C.free(unsafe.Pointer(cFile))

	result := C.xplainit_on_function_enter(x.handle, cName, cFile, C.uint32_t(line))
	return result != 0
}

// OnFunctionExit records a function exit event through the C FFI.
//
// The event is only recorded when tracing is enabled. Returns true if the
// event was recorded.
func (x *Xplainit) OnFunctionExit(name, file string, line int) bool {
	if x.handle == nil {
		return false
	}

	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))
	cFile := C.CString(file)
	defer C.free(unsafe.Pointer(cFile))

	result := C.xplainit_on_function_exit(x.handle, cName, cFile, C.uint32_t(line))
	return result != 0
}

// OnException records an exception/error event through the C FFI.
//
// The event is only recorded when tracing is enabled. errType is the error
// type (e.g. the Go error's dynamic type), message is the error text.
// Returns true if the event was recorded.
func (x *Xplainit) OnException(errType, message, file string, line int) bool {
	if x.handle == nil {
		return false
	}

	cType := C.CString(errType)
	defer C.free(unsafe.Pointer(cType))
	cMessage := C.CString(message)
	defer C.free(unsafe.Pointer(cMessage))
	cFile := C.CString(file)
	defer C.free(unsafe.Pointer(cFile))

	result := C.xplainit_on_exception(x.handle, cType, cMessage, cFile, C.uint32_t(line))
	return result != 0
}

// Trace records a function-enter event immediately and returns a closure that
// records the matching function-exit event when called. It is meant to be used
// with defer, which is Go's idiomatic instrumentation mechanism:
//
//	func myFunc(x *xplainit.Xplainit) {
//	    defer x.Trace("myFunc", "file.go", 12)()
//	    // ... function body ...
//	}
//
// The trailing () invokes Trace immediately (recording the enter event) and
// defers the returned closure, so the exit event fires when myFunc returns.
//
// Go does not provide an automatic tracing hook like Python's sys.settrace,
// so this manual/defer wrapper is the supported way to capture events from Go
// code. Fully automatic instrumentation would require a debugger such as delve
// or source-level code generation, which is future work.
func (x *Xplainit) Trace(name, file string, line int) func() {
	x.OnFunctionEnter(name, file, line)
	return func() {
		x.OnFunctionExit(name, file, line)
	}
}

// Version returns the Xplainit version
func Version() string {
	cStr := C.xplainit_version()
	return C.GoString(cStr)
}
