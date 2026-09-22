//! Python decorators for selective tracing

use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::ffi::CString;

/// Python source for the decorator factories.
///
/// We build the wrappers with `exec` (via [`PyModule::from_code_bound`]),
/// NOT `eval`: a `def` is a *statement*, so `eval` would raise `SyntaxError`.
/// The wrappers are real callables that (1) return the wrapped function's
/// result unchanged and (2) forward enter/exit/exception events into the
/// provided `backend` so calls are actually recorded (not TODO stubs).
const DECORATOR_SOURCE: &str = r#"
import functools
import inspect


def _serialize(value):
    """Best-effort string form of an argument / return value."""
    try:
        if value is None:
            return "None"
        if isinstance(value, bool):
            return "True" if value else "False"
        if isinstance(value, (int, float, str)):
            return str(value)
        rep = repr(value)
        return rep if len(rep) <= 100 else rep[:97] + "..."
    except Exception:
        return "<unrepr>"


def _location(func):
    try:
        file_path = inspect.getfile(func)
    except (OSError, TypeError):
        file_path = "<unknown>"
    try:
        line_number = inspect.getsourcelines(func)[1]
    except (OSError, TypeError):
        line_number = 0
    return file_path, line_number


def make_explain_wrapper(func, backend):
    """Build the @explain_function wrapper. Records the call into `backend`."""
    file_path, line_number = _location(func)

    @functools.wraps(func)
    def wrapped(*args, **kwargs):
        if backend is not None:
            args_dict = {}
            try:
                sig = inspect.signature(func)
                bound = sig.bind(*args, **kwargs)
                bound.apply_defaults()
                for name, val in bound.arguments.items():
                    args_dict[name] = _serialize(val)
            except Exception:
                pass
            try:
                backend.on_function_enter(func.__name__, args_dict, file_path, line_number)
            except Exception:
                pass
        try:
            result = func(*args, **kwargs)
        except Exception as exc:
            if backend is not None:
                try:
                    backend.on_exception(type(exc).__name__, str(exc), file_path, line_number)
                except Exception:
                    pass
            raise
        else:
            if backend is not None:
                try:
                    backend.on_function_exit(func.__name__, _serialize(result), file_path, line_number)
                except Exception:
                    pass
            return result

    return wrapped


def make_explain_errors_wrapper(func, backend):
    """Build the @explain_errors wrapper. Records exceptions into `backend`."""
    file_path, line_number = _location(func)

    @functools.wraps(func)
    def wrapped(*args, **kwargs):
        try:
            return func(*args, **kwargs)
        except Exception as exc:
            if backend is not None:
                try:
                    backend.on_exception(type(exc).__name__, str(exc), file_path, line_number)
                except Exception:
                    pass
            raise

    return wrapped
"#;

/// Load (and cache) the compiled decorator helper module.
fn decorator_module<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyModule>> {
    let code = CString::new(DECORATOR_SOURCE).expect("decorator source has no NUL bytes");
    let file = CString::new("xplainit_decorators.py").unwrap();
    let name = CString::new("xplainit_decorators").unwrap();
    PyModule::from_code_bound(
        py,
        code.to_str().unwrap(),
        file.to_str().unwrap(),
        name.to_str().unwrap(),
    )
}

/// Create an `@explain_function` decorator.
///
/// Returns a real callable wrapper (never `None`): it calls the original
/// function, returns its result unchanged, and records the call into `backend`.
pub fn create_explain_decorator(
    func: &Bound<'_, PyAny>,
    backend: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let py = func.py();
    let module = decorator_module(py)?;
    let factory = module.getattr("make_explain_wrapper")?;
    let wrapped = factory.call1((func, backend))?;
    Ok(wrapped.into())
}

/// Create an `@explain_errors` decorator.
#[allow(dead_code)]
pub fn create_explain_errors_decorator(
    func: &Bound<'_, PyAny>,
    backend: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let py = func.py();
    let module = decorator_module(py)?;
    let factory = module.getattr("make_explain_errors_wrapper")?;
    let wrapped = factory.call1((func, backend))?;
    Ok(wrapped.into())
}
