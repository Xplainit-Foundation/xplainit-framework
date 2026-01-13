"""
Python sys.settrace() Integration for Xplainit

This module provides automatic runtime tracing for Python code using
the sys.settrace() mechanism.
"""

import sys
import os
import inspect
from typing import Any, Optional, Dict, Callable


class XplainitTracer:
    """
    Automatic Python tracer using sys.settrace().
    
    This tracer hooks into Python's execution model to capture:
    - Function calls with arguments
    - Function returns with return values
    - Line execution
    - Exception handling
    
    It filters out stdlib/site-packages to focus on user code.
    """
    
    def __init__(self, rust_backend, trace_lines=False, capture_locals=False):
        """
        Initialize the tracer.
        
        Args:
            rust_backend: The Rust backend object (Xplainit instance)
            trace_lines: Enable line-level tracing (can be expensive)
            capture_locals: Capture local variable values (very expensive)
        """
        self.rust_backend = rust_backend
        self.enabled = False
        self.trace_lines = trace_lines
        self.capture_locals = capture_locals
        self._previous_trace = None
        self._traced_files = set()
        self._ignored_files = set()
        
    def should_trace_file(self, filename: str) -> bool:
        """
        Determine if a file should be traced.
        
        Only traces user code, not Python stdlib or site-packages.
        
        Args:
            filename: The file path to check
            
        Returns:
            True if the file should be traced, False otherwise
        """
        # Cache decision
        if filename in self._traced_files:
            return True
        if filename in self._ignored_files:
            return False
        
        # Don't trace special files
        if filename.startswith('<'):
            self._ignored_files.add(filename)
            return False
        
        # Don't trace site-packages
        if 'site-packages' in filename:
            self._ignored_files.add(filename)
            return False
        
        # Don't trace Python stdlib
        if filename.startswith(sys.prefix):
            self._ignored_files.add(filename)
            return False
        
        # Check common stdlib paths
        for path in ['/usr/lib/python', '/usr/local/lib/python', 
                     'C:\\Python', 'C:\\Program Files\\Python']:
            if filename.startswith(path):
                self._ignored_files.add(filename)
                return False
        
        # Must exist and be a real file
        if not os.path.exists(filename):
            self._ignored_files.add(filename)
            return False
        
        # It's user code!
        self._traced_files.add(filename)
        return True
    
    def _extract_args(self, frame) -> Dict[str, Any]:
        """
        Extract function arguments from a frame.
        
        Args:
            frame: The Python frame object
            
        Returns:
            Dictionary mapping argument names to values
        """
        args = {}
        code = frame.f_code
        arg_count = code.co_argcount
        arg_names = code.co_varnames[:arg_count]
        
        for name in arg_names:
            if name in frame.f_locals:
                value = frame.f_locals[name]
                # Convert to string representation
                try:
                    args[name] = self._serialize_value(value)
                except Exception:
                    args[name] = f"<{type(value).__name__}>"
        
        return args
    
    def _serialize_value(self, value: Any) -> str:
        """
        Convert a Python value to a serializable string.
        
        Args:
            value: The value to serialize
            
        Returns:
            String representation of the value
        """
        # Handle None
        if value is None:
            return "None"
        
        # Handle primitives
        if isinstance(value, (bool, int, float, str)):
            return repr(value)
        
        # Handle collections (limited depth)
        if isinstance(value, (list, tuple)):
            if len(value) <= 5:
                items = [self._serialize_value(v) for v in value]
                bracket = '[' if isinstance(value, list) else '('
                close = ']' if isinstance(value, list) else ')'
                return f"{bracket}{', '.join(items)}{close}"
            else:
                return f"<{type(value).__name__} of length {len(value)}>"
        
        if isinstance(value, dict):
            if len(value) <= 5:
                items = [f"{k}: {self._serialize_value(v)}" 
                        for k, v in list(value.items())[:5]]
                return f"{{{', '.join(items)}}}"
            else:
                return f"<dict with {len(value)} items>"
        
        # Everything else gets type name
        return f"<{type(value).__name__}>"
    
    def trace_function(self, frame, event: str, arg):
        """
        Main sys.settrace callback function.
        
        This is called by Python for every execution event.
        
        Args:
            frame: The current frame object
            event: The event type ('call', 'return', 'line', 'exception')
            arg: Additional argument (depends on event type)
            
        Returns:
            The trace function to use (self for continued tracing)
        """
        if not self.enabled:
            return None
        
        # Get frame information
        code = frame.f_code
        filename = code.co_filename
        line = frame.f_lineno
        function_name = code.co_name
        
        # Filter out files we don't want to trace
        if not self.should_trace_file(filename):
            return None
        
        try:
            if event == 'call':
                # Function entry
                args = self._extract_args(frame)
                self.rust_backend.on_function_enter(
                    function_name, args, filename, line
                )
            
            elif event == 'return':
                # Function exit
                return_value = self._serialize_value(arg) if arg is not None else "None"
                self.rust_backend.on_function_exit(
                    function_name, return_value, filename, line
                )
            
            elif event == 'line':
                # Line execution - only if enabled
                if self.trace_lines:
                    local_vars = {}
                    if self.capture_locals:
                        # Capture local variables if enabled
                        for var_name, var_value in frame.f_locals.items():
                            if not var_name.startswith('__'):
                                try:
                                    local_vars[var_name] = self._serialize_value(var_value)
                                except Exception:
                                    local_vars[var_name] = f"<{type(var_value).__name__}>"
                    
                    # Record line execution
                    self.rust_backend.on_line_execute(filename, line, local_vars)
            
            elif event == 'exception':
                # Line execution (only trace if verbose)
                # We'll skip this for now to reduce overhead
                pass
            
            elif event == 'exception':
                # Exception occurred
                exc_type, exc_value, exc_tb = arg
                self.rust_backend.on_exception(
                    exc_type.__name__, str(exc_value), filename, line
                )
        
        except Exception as e:
            # Don't let tracer errors break the program
            print(f"Warning: Xplainit tracer error: {e}", file=sys.stderr)
        
        # Return self to continue tracing this frame
        return self.trace_function
    
    def enable(self):
        """Enable automatic tracing."""
        if not self.enabled:
            self.enabled = True
            self._previous_trace = sys.gettrace()
            sys.settrace(self.trace_function)
    
    def disable(self):
        """Disable automatic tracing."""
        if self.enabled:
            self.enabled = False
            sys.settrace(self._previous_trace)
            self._previous_trace = None
    
    def __enter__(self):
        """Context manager entry."""
        self.enable()
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.disable()
        return False
