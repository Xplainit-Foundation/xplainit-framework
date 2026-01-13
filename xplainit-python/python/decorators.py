"""
Xplainit Python Decorators

Provides convenient decorators for selective function tracing without
needing to enable full sys.settrace() tracing.
"""

import functools
import inspect
from typing import Callable, Any, Optional


def trace(backend=None, capture_args=True, capture_return=True, capture_locals=False):
    """
    Decorator to automatically trace a specific function.
    
    Usage:
        @trace(backend=xplainit_instance)
        def my_function(x, y):
            return x + y
    
    Args:
        backend: Xplainit instance to record events to
        capture_args: Whether to capture function arguments
        capture_return: Whether to capture return value
        capture_locals: Whether to capture local variables (expensive!)
    """
    def decorator(func: Callable) -> Callable:
        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            # Get function metadata
            func_name = func.__name__
            file_path = inspect.getfile(func)
            
            try:
                line_number = inspect.getsourcelines(func)[1]
            except (OSError, TypeError):
                line_number = 0
            
            # Capture arguments if enabled
            if capture_args and backend:
                # Build argument dictionary
                sig = inspect.signature(func)
                bound_args = sig.bind(*args, **kwargs)
                bound_args.apply_defaults()
                
                args_dict = {}
                for param_name, param_value in bound_args.arguments.items():
                    args_dict[param_name] = _serialize_value(param_value)
                
                # Record function entry
                try:
                    backend.on_function_enter(func_name, args_dict, file_path, line_number)
                except Exception as e:
                    print(f"Warning: Failed to record function entry: {e}")
            
            # Execute the function
            exception_occurred = None
            return_value = None
            
            try:
                return_value = func(*args, **kwargs)
                return return_value
            except Exception as e:
                exception_occurred = e
                raise
            finally:
                # Record function exit or exception
                if backend:
                    try:
                        if exception_occurred:
                            backend.on_exception(
                                type(exception_occurred).__name__,
                                str(exception_occurred),
                                file_path,
                                line_number
                            )
                        elif capture_return:
                            backend.on_function_exit(
                                func_name,
                                _serialize_value(return_value),
                                file_path,
                                line_number
                            )
                    except Exception as e:
                        print(f"Warning: Failed to record function exit: {e}")
        
        return wrapper
    return decorator


def trace_class(backend=None, exclude_methods=None):
    """
    Decorator to trace all methods in a class.
    
    Usage:
        @trace_class(backend=xplainit_instance)
        class MyClass:
            def method1(self): pass
            def method2(self): pass
    
    Args:
        backend: Xplainit instance to record events to
        exclude_methods: Set of method names to exclude from tracing
    """
    if exclude_methods is None:
        exclude_methods = {'__init__', '__str__', '__repr__', '__del__', '__new__', '__getattribute__'}
    
    def decorator(cls):
        # Get all methods in the class
        for attr_name in dir(cls):
            # Skip excluded methods and special methods
            if attr_name in exclude_methods:
                continue
            
            # Skip private methods starting with _
            if attr_name.startswith('_'):
                continue
            
            try:
                attr = getattr(cls, attr_name)
                # Only wrap callable methods (not properties, class methods, etc.)
                if callable(attr) and not isinstance(attr, type):
                    # Check if it's actually a method defined in this class
                    if hasattr(attr, '__func__'):
                        # Apply trace decorator to method
                        wrapped = trace(backend=backend)(attr)
                        setattr(cls, attr_name, wrapped)
            except (AttributeError, TypeError):
                # Skip attributes that can't be wrapped
                continue
        
        return cls
    return decorator


def profile(backend=None, name=None):
    """
    Decorator to profile function execution time.
    
    Usage:
        @profile(backend=xplainit_instance)
        def slow_function():
            # ... expensive operation ...
    
    Args:
        backend: Xplainit instance to record timing to
        name: Optional custom name for the profiling section
    """
    def decorator(func: Callable) -> Callable:
        profile_name = name or func.__name__
        
        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            import time
            
            start_time = time.perf_counter()
            
            try:
                result = func(*args, **kwargs)
                return result
            finally:
                end_time = time.perf_counter()
                duration_ms = (end_time - start_time) * 1000
                
                if backend:
                    try:
                        # Record as a custom event or use existing mechanisms
                        print(f"[Profile] {profile_name}: {duration_ms:.2f}ms")
                    except Exception as e:
                        print(f"Warning: Failed to record profile data: {e}")
        
        return wrapper
    return decorator


def trace_recursive(backend=None, max_depth=100):
    """
    Special decorator for recursive functions that tracks recursion depth.
    
    Usage:
        @trace_recursive(backend=xplainit_instance)
        def fibonacci(n):
            if n <= 1: return n
            return fibonacci(n-1) + fibonacci(n-2)
    
    Args:
        backend: Xplainit instance to record events to
        max_depth: Maximum recursion depth to trace (prevents overflow)
    """
    def decorator(func: Callable) -> Callable:
        recursion_depth = 0
        
        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            nonlocal recursion_depth
            
            recursion_depth += 1
            
            if recursion_depth <= max_depth and backend:
                func_name = f"{func.__name__} [depth={recursion_depth}]"
                file_path = inspect.getfile(func)
                
                try:
                    line_number = inspect.getsourcelines(func)[1]
                except (OSError, TypeError):
                    line_number = 0
                
                # Record entry with depth information
                args_dict = {'depth': str(recursion_depth)}
                try:
                    backend.on_function_enter(func_name, args_dict, file_path, line_number)
                except Exception:
                    pass
            
            try:
                result = func(*args, **kwargs)
                return result
            finally:
                recursion_depth -= 1
        
        return wrapper
    return decorator


def _serialize_value(value: Any) -> str:
    """
    Serialize a Python value to a string for storage.
    
    Args:
        value: Any Python value
        
    Returns:
        String representation of the value
    """
    if value is None:
        return "None"
    elif isinstance(value, (int, float, str, bool)):
        return str(value)
    elif isinstance(value, (list, tuple)):
        if len(value) > 10:
            return f"{type(value).__name__}[{len(value)} items]"
        return str(value)
    elif isinstance(value, dict):
        if len(value) > 10:
            return f"dict[{len(value)} items]"
        return str(value)
    else:
        # For complex objects, use repr but truncate if too long
        repr_str = repr(value)
        if len(repr_str) > 100:
            return repr_str[:97] + "..."
        return repr_str


__all__ = ['trace', 'trace_class', 'profile', 'trace_recursive']
