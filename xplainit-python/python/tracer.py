"""
Xplainit Automatic Tracer using sys.settrace()

This module provides automatic tracing by hooking into Python's
sys.settrace() mechanism. It captures function calls, returns,
exceptions, and line executions automatically.
"""

import sys
import inspect
import threading
from typing import Optional, Dict, Any, Set
import time


class AutoTracer:
    """
    Automatic tracer that integrates with Python's sys.settrace().
    
    This class provides the bridge between Python's tracing mechanism
    and the Rust backend (xplainit.Xplainit instance).
    
    Usage:
        tracer = AutoTracer(backend=xplainit_instance)
        tracer.start()
        # ... Python code runs automatically traced ...
        tracer.stop()
    """
    
    def __init__(self, backend=None, 
                 trace_calls: bool = True,
                 trace_returns: bool = True,
                 trace_exceptions: bool = True,
                 trace_lines: bool = False,  # Expensive!
                 exclude_modules: Optional[Set[str]] = None,
                 max_depth: int = 100):
        """
        Initialize the automatic tracer.
        
        Args:
            backend: xplainit.Xplainit instance (Rust backend)
            trace_calls: Trace function calls (default: True)
            trace_returns: Trace function returns (default: True)
            trace_exceptions: Trace exceptions (default: True)
            trace_lines: Trace line executions (default: False - expensive!)
            exclude_modules: Set of module names to exclude from tracing
            max_depth: Maximum call depth to trace (prevent stack overflow)
        """
        self.backend = backend
        self.trace_calls = trace_calls
        self.trace_returns = trace_returns
        self.trace_exceptions = trace_exceptions
        self.trace_lines = trace_lines
        self.exclude_modules = exclude_modules or set()
        self.max_depth = max_depth
        
        # Thread-local storage for recursion depth
        self._thread_local = threading.local()
        
        # Track if tracing is active
        self._active = False
        self._previous_trace = None
        
    def start(self):
        """Start automatic tracing by installing sys.settrace() hook."""
        if self._active:
            return
        
        self._active = True
        self._previous_trace = sys.gettrace()
        sys.settrace(self._trace_callback)
        
    def stop(self):
        """Stop automatic tracing by removing sys.settrace() hook."""
        if not self._active:
            return
        
        self._active = False
        sys.settrace(self._previous_trace)
        self._previous_trace = None
        
    def _get_depth(self) -> int:
        """Get current call depth for this thread."""
        if not hasattr(self._thread_local, 'depth'):
            self._thread_local.depth = 0
        return self._thread_local.depth
    
    def _set_depth(self, depth: int):
        """Set current call depth for this thread."""
        self._thread_local.depth = depth
        
    def _should_trace(self, frame) -> bool:
        """
        Determine if we should trace this frame.
        
        Args:
            frame: Python frame object
            
        Returns:
            True if we should trace this frame, False otherwise
        """
        # Check depth limit
        if self._get_depth() > self.max_depth:
            return False
        
        # Get module name
        module_name = frame.f_globals.get('__name__', '')
        
        # Exclude standard library and third-party modules
        if module_name in self.exclude_modules:
            return False
        
        # Exclude common standard library modules
        stdlib_prefixes = [
            'sys', 'os', 'io', 'abc', 'codecs', 'encodings',
            'importlib', 'collections', 'typing', 'functools',
            'threading', 'weakref', 'contextlib', 'traceback',
            'tokenize', 'token', 'linecache', 'dis', 'opcode'
        ]
        
        for prefix in stdlib_prefixes:
            if module_name.startswith(prefix):
                return False
        
        # Exclude xplainit's own code to prevent infinite recursion
        if 'xplainit' in module_name or 'tracer' in module_name:
            return False
        
        return True
    
    def _trace_callback(self, frame, event: str, arg: Any):
        """
        Main trace callback function called by Python interpreter.
        
        This is the core hook that Python calls for every event.
        
        Args:
            frame: Python frame object
            event: Event type ('call', 'return', 'line', 'exception')
            arg: Event-specific argument
            
        Returns:
            Local trace function or None
        """
        # Skip if not active
        if not self._active:
            return None
        
        # Skip if no backend
        if self.backend is None:
            return None
        
        # Check if we should trace this frame
        if not self._should_trace(frame):
            return None
        
        try:
            # Handle different event types
            if event == 'call' and self.trace_calls:
                return self._handle_call(frame)
            elif event == 'return' and self.trace_returns:
                return self._handle_return(frame, arg)
            elif event == 'exception' and self.trace_exceptions:
                return self._handle_exception(frame, arg)
            elif event == 'line' and self.trace_lines:
                return self._handle_line(frame)
        except Exception as e:
            # Never let tracing errors crash the program
            # In production, we'd log this
            pass
        
        return None
    
    def _handle_call(self, frame):
        """
        Handle function call event.
        
        Args:
            frame: Python frame object
            
        Returns:
            Local trace function (self._trace_callback)
        """
        # Increment depth
        depth = self._get_depth()
        self._set_depth(depth + 1)
        
        # Extract function information
        func_name = frame.f_code.co_name
        filename = frame.f_code.co_filename
        line_number = frame.f_lineno
        
        # Extract arguments
        args_dict = {}
        try:
            # Get argument names
            arg_names = frame.f_code.co_varnames[:frame.f_code.co_argcount]
            
            # Get argument values from frame locals
            for arg_name in arg_names:
                if arg_name in frame.f_locals:
                    value = frame.f_locals[arg_name]
                    args_dict[arg_name] = self._serialize_value(value)
        except Exception:
            pass
        
        # Call Rust backend
        try:
            self.backend.on_function_enter(
                func_name,
                args_dict,
                filename,
                line_number
            )
        except Exception:
            pass
        
        # Return local trace function
        return self._trace_callback
    
    def _handle_return(self, frame, return_value):
        """
        Handle function return event.
        
        Args:
            frame: Python frame object
            return_value: Value being returned (or None)
            
        Returns:
            None
        """
        # Decrement depth
        depth = self._get_depth()
        self._set_depth(max(0, depth - 1))
        
        # Extract function information
        func_name = frame.f_code.co_name
        filename = frame.f_code.co_filename
        line_number = frame.f_lineno
        
        # Serialize return value
        return_str = self._serialize_value(return_value)
        
        # Call Rust backend
        try:
            self.backend.on_function_exit(
                func_name,
                return_str,
                filename,
                line_number
            )
        except Exception:
            pass
        
        return None
    
    def _handle_exception(self, frame, exc_info):
        """
        Handle exception event.
        
        Args:
            frame: Python frame object
            exc_info: Tuple of (exc_type, exc_value, exc_traceback)
            
        Returns:
            None
        """
        if exc_info is None:
            return None
        
        exc_type, exc_value, exc_traceback = exc_info
        
        # Extract exception information
        exc_type_name = exc_type.__name__ if exc_type else 'Unknown'
        exc_message = str(exc_value) if exc_value else ''
        filename = frame.f_code.co_filename
        line_number = frame.f_lineno
        
        # Call Rust backend
        try:
            self.backend.on_exception(
                exc_type_name,
                exc_message,
                filename,
                line_number
            )
        except Exception:
            pass
        
        return None
    
    def _handle_line(self, frame):
        """
        Handle line execution event.
        
        WARNING: This is extremely expensive! Only use for detailed debugging.
        
        Args:
            frame: Python frame object
            
        Returns:
            None
        """
        # For now, we skip line-level tracing due to overhead
        # In the future, we could implement this for debug verbosity
        return None
    
    @staticmethod
    def _serialize_value(value: Any) -> str:
        """
        Serialize a Python value to string for Rust backend.
        
        Args:
            value: Any Python value
            
        Returns:
            String representation
        """
        if value is None:
            return "None"
        elif isinstance(value, bool):
            return "True" if value else "False"
        elif isinstance(value, int):
            return str(value)
        elif isinstance(value, float):
            return str(value)
        elif isinstance(value, str):
            # Return quoted string
            return f"'{value}'"
        elif isinstance(value, (list, tuple)):
            # Limit size to prevent huge strings
            if len(value) > 10:
                return f"{type(value).__name__}[{len(value)} items]"
            return str(value)
        elif isinstance(value, dict):
            # Limit size
            if len(value) > 10:
                return f"dict[{len(value)} items]"
            return str(value)
        else:
            # For objects, use repr but truncate
            try:
                repr_str = repr(value)
                if len(repr_str) > 100:
                    return repr_str[:97] + "..."
                return repr_str
            except Exception:
                return f"<{type(value).__name__}>"


def enable_tracing(backend=None, **kwargs):
    """
    Convenience function to enable global tracing.
    
    Usage:
        import xplainit
        xplainit.enable_tracing()
        
        # ... code runs with automatic tracing ...
        
        xplainit.disable_tracing()
    
    Args:
        backend: xplainit.Xplainit instance
        **kwargs: Additional arguments passed to AutoTracer
    """
    global _global_tracer
    
    if backend is None:
        # Create default backend if not provided
        import xplainit
        backend = xplainit.Xplainit()
    
    _global_tracer = AutoTracer(backend=backend, **kwargs)
    _global_tracer.start()


def disable_tracing():
    """
    Convenience function to disable global tracing.
    """
    global _global_tracer
    
    if _global_tracer is not None:
        _global_tracer.stop()
        _global_tracer = None


# Global tracer instance
_global_tracer: Optional[AutoTracer] = None


__all__ = ['AutoTracer', 'enable_tracing', 'disable_tracing']
