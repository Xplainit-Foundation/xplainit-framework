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
	"encoding/json"
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

// Version returns the Xplainit version
func Version() string {
	cStr := C.xplainit_version()
	return C.GoString(cStr)
}
