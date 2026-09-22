//! Python bindings for Xplainit Framework
//! Provides sys.settrace() integration for runtime code explanation

// The pyo3 `#[pymethods]`/`#[pyfunction]` macros generate wrapper code that
// converts the returned `PyResult` error type via `.into()`, which clippy
// flags as `useless_conversion` (PyErr -> PyErr). The conversion lives in
// generated code we do not control, so silence the lint crate-wide.
#![allow(clippy::useless_conversion)]

use parking_lot::RwLock;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;
use std::sync::Arc;
use xplainit_core::*;

mod decorators;
mod tracer;

use tracer::PythonTracer;

/// Python module initialization
#[pymodule]
fn xplainit(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Xplainit>()?;
    m.add_class::<XplainitContext>()?;
    m.add_class::<AutoTracer>()?;
    m.add_function(wrap_pyfunction!(py_enable, m)?)?;
    m.add_function(wrap_pyfunction!(py_disable, m)?)?;
    m.add_function(wrap_pyfunction!(py_is_enabled, m)?)?;
    m.add_function(wrap_pyfunction!(explain_function, m)?)?;
    m.add_function(wrap_pyfunction!(explain_backend_py, m)?)?;
    m.add_function(wrap_pyfunction!(get_last_explanation, m)?)?;
    m.add_function(wrap_pyfunction!(auto_trace, m)?)?;
    m.add_function(wrap_pyfunction!(disable_auto_trace, m)?)?;

    Ok(())
}

/// Global Xplainit instance
static GLOBAL_INSTANCE: once_cell::sync::Lazy<Arc<RwLock<Option<PythonTracer>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(None)));

/// Main Xplainit class for Python
#[pyclass]
struct Xplainit {
    tracer: Arc<RwLock<PythonTracer>>,
}

#[pymethods]
impl Xplainit {
    #[new]
    #[pyo3(signature = (enabled=true, verbosity="normal", output="stdout"))]
    fn new(enabled: bool, verbosity: &str, output: &str) -> PyResult<Self> {
        let config = create_config(verbosity, output);
        let tracer = PythonTracer::new(config, enabled);

        Ok(Self {
            tracer: Arc::new(RwLock::new(tracer)),
        })
    }

    /// Enable tracing
    fn enable(&self) {
        self.tracer.write().enable();
    }

    /// Disable tracing
    fn disable(&self) {
        self.tracer.write().disable();
    }

    /// Check if enabled
    fn is_enabled(&self) -> bool {
        self.tracer.read().is_enabled()
    }

    /// Get all captured events as JSON
    fn get_events(&self) -> String {
        self.tracer.read().get_events_json()
    }

    /// Get last explanation
    fn get_last_explanation(&self) -> String {
        self.tracer.read().get_last_explanation()
    }

    /// Clear all events
    fn clear(&self) {
        self.tracer.write().clear();
    }

    /// Set verbosity level
    fn set_verbosity(&self, level: &str) {
        self.tracer.write().set_verbosity(level);
    }

    /// Get statistics
    fn get_stats(&self) -> String {
        self.tracer.read().get_stats()
    }

    /// Get explanations for all captured events
    #[pyo3(signature = (verbosity=None))]
    fn get_explanations(&self, verbosity: Option<&str>) -> String {
        self.tracer.read().get_explanations(verbosity)
    }

    /// Print explanations in real-time to console
    #[pyo3(signature = (verbosity=None))]
    fn print_explanations(&self, verbosity: Option<&str>) {
        let verb = verbosity.unwrap_or("normal");
        let explanations = self.tracer.read().get_explanations(Some(verb));
        println!("{}", explanations);
    }

    // ===== sys.settrace() callback methods =====

    /// Called when a function is entered (from Python tracer)
    fn on_function_enter(
        &self,
        name: String,
        args: &Bound<'_, PyDict>,
        filename: String,
        line: usize,
    ) -> PyResult<()> {
        // Convert Python dict to HashMap<String, Value>
        let mut rust_args = HashMap::new();
        for (key, value) in args.iter() {
            if let Ok(key_str) = key.extract::<String>() {
                if let Ok(value_str) = value.extract::<String>() {
                    let val = tracer::parse_python_value(&value_str);
                    rust_args.insert(key_str, val);
                }
            }
        }

        self.tracer
            .write()
            .record_function_enter(name, rust_args, filename, line);
        Ok(())
    }

    /// Called when a function exits (from Python tracer)
    fn on_function_exit(
        &self,
        name: String,
        return_value: String,
        filename: String,
        line: usize,
    ) -> PyResult<()> {
        let val = tracer::parse_python_value(&return_value);

        self.tracer
            .write()
            .record_function_exit(name, Some(val), filename, line);
        Ok(())
    }

    /// Called when an exception occurs (from Python tracer)
    fn on_exception(
        &self,
        exc_type: String,
        exc_message: String,
        filename: String,
        line: usize,
    ) -> PyResult<()> {
        self.tracer
            .write()
            .record_exception(exc_type, exc_message, filename, line);
        Ok(())
    }

    /// Called when a line is executed (from Python tracer) - optional
    fn on_line_execute(
        &self,
        _filename: String,
        _line: usize,
        _locals: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        // For now, we don't record line events (too much overhead)
        // This can be enabled for debug verbosity level
        Ok(())
    }
}

/// Context manager for scoped tracing
#[pyclass]
struct XplainitContext {
    tracer: Arc<RwLock<PythonTracer>>,
}

#[pymethods]
impl XplainitContext {
    #[new]
    #[pyo3(signature = (enabled=true, verbosity="normal"))]
    fn new(enabled: bool, verbosity: &str) -> Self {
        let config = create_config(verbosity, "stdout");
        let tracer = PythonTracer::new(config, enabled);

        Self {
            tracer: Arc::new(RwLock::new(tracer)),
        }
    }

    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        // Simply return self for the context manager protocol
        slf
    }

    #[pyo3(signature = (_exc_type=None, _exc_value=None, _traceback=None))]
    fn __exit__(
        &self,
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_value: Option<&Bound<'_, PyAny>>,
        _traceback: Option<&Bound<'_, PyAny>>,
    ) -> bool {
        // Disable tracing on exit
        self.tracer.write().disable();
        false // Don't suppress exceptions
    }

    fn get_events(&self) -> String {
        self.tracer.read().get_events_json()
    }
}

// ===== Module-level functions =====

/// Enable global tracing
#[pyfunction]
fn py_enable(_py: Python) {
    let mut global = GLOBAL_INSTANCE.write();
    if global.is_none() {
        let config = create_config("normal", "stdout");
        *global = Some(PythonTracer::new(config, true));
    }

    if let Some(tracer) = global.as_mut() {
        tracer.enable();
    }
}

/// Disable global tracing
#[pyfunction]
fn py_disable(_py: Python) {
    if let Some(tracer) = GLOBAL_INSTANCE.write().as_mut() {
        tracer.disable();
    }
}

/// Check if global tracing is enabled
#[pyfunction]
fn py_is_enabled() -> bool {
    GLOBAL_INSTANCE
        .read()
        .as_ref()
        .map(|t| t.is_enabled())
        .unwrap_or(false)
}

/// Process-wide backend used by the bare `@explain_function` decorator so the
/// calls it wraps are actually recorded somewhere retrievable.
static EXPLAIN_BACKEND: once_cell::sync::OnceCell<Py<Xplainit>> = once_cell::sync::OnceCell::new();

/// Get (lazily creating) the shared backend used by `@explain_function`.
fn explain_backend(py: Python<'_>) -> PyResult<Py<Xplainit>> {
    if let Some(backend) = EXPLAIN_BACKEND.get() {
        return Ok(backend.clone_ref(py));
    }
    let backend = Py::new(py, Xplainit::new(true, "normal", "stdout")?)?;
    // If another thread raced us, keep the first one stored.
    let _ = EXPLAIN_BACKEND.set(backend.clone_ref(py));
    Ok(EXPLAIN_BACKEND.get().unwrap().clone_ref(py))
}

/// Decorator to explain (trace) a specific function.
///
/// Returns a real callable wrapper. Errors are propagated (never swallowed
/// into `None`, which would turn the decorated function into a non-callable).
#[pyfunction]
fn explain_function(py: Python<'_>, func: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let backend = explain_backend(py)?;
    let backend_bound = backend.bind(py);
    decorators::create_explain_decorator(func, backend_bound.as_any())
}

/// Return the shared backend that `@explain_function` records into.
///
/// Lets callers inspect the events captured by decorated functions, e.g.
/// `json.loads(xplainit.explain_backend().get_events())`.
#[pyfunction]
#[pyo3(name = "explain_backend")]
fn explain_backend_py(py: Python<'_>) -> PyResult<Py<Xplainit>> {
    explain_backend(py)
}

/// Get last explanation from global instance
#[pyfunction]
fn get_last_explanation() -> String {
    GLOBAL_INSTANCE
        .read()
        .as_ref()
        .map(|t| t.get_last_explanation())
        .unwrap_or_else(|| "No explanations available".to_string())
}

/// Automatic tracer using sys.settrace().
///
/// Unlike the earlier no-op version, this class installs a real
/// `sys.settrace()` hook by delegating to the pure-Python
/// `xplainit.python.tracer.AutoTracer`, wiring it to an internal
/// [`Xplainit`] backend. Function enter/exit and exception events are
/// therefore captured for real and are retrievable via [`AutoTracer::get_events`].
#[pyclass]
struct AutoTracer {
    /// The Rust backend that actually records events. Held as a `Py<Xplainit>`
    /// so it can be handed to the Python tracer as its `backend`.
    backend: Py<Xplainit>,
    /// The live `xplainit.python.tracer.AutoTracer` instance while tracing is
    /// active (installs/removes the `sys.settrace()` hook). `None` when stopped.
    py_tracer: RwLock<Option<PyObject>>,
}

#[pymethods]
impl AutoTracer {
    #[new]
    #[pyo3(signature = (_backend=None))]
    fn new(py: Python<'_>, _backend: Option<&Bound<'_, PyAny>>) -> PyResult<Self> {
        let backend = Py::new(py, Xplainit::new(true, "normal", "stdout")?)?;

        Ok(Self {
            backend,
            py_tracer: RwLock::new(None),
        })
    }

    /// Start automatic tracing.
    ///
    /// Installs a real `sys.settrace()` hook via the Python tracer, forwarding
    /// captured events into this instance's backend.
    fn start(&self, py: Python<'_>) -> PyResult<()> {
        // Already tracing? Make start() idempotent.
        if self.py_tracer.read().is_some() {
            return Ok(());
        }

        self.backend.borrow(py).enable();

        let tracer_module = py.import_bound("xplainit.python.tracer")?;
        let auto_tracer_class = tracer_module.getattr("AutoTracer")?;
        // Pass our Rust backend as the `backend=` keyword so the Python tracer
        // forwards on_function_enter/exit/exception into it.
        let kwargs = PyDict::new_bound(py);
        kwargs.set_item("backend", self.backend.clone_ref(py))?;
        let py_tracer = auto_tracer_class.call((), Some(&kwargs))?;
        py_tracer.call_method0("start")?;

        *self.py_tracer.write() = Some(py_tracer.into());
        Ok(())
    }

    /// Stop automatic tracing and remove the `sys.settrace()` hook.
    fn stop(&self, py: Python<'_>) -> PyResult<()> {
        let tracer = self.py_tracer.write().take();
        if let Some(tracer) = tracer {
            tracer.call_method0(py, "stop")?;
        }
        self.backend.borrow(py).disable();
        Ok(())
    }

    /// Check if tracing is active
    fn is_active(&self) -> bool {
        self.py_tracer.read().is_some()
    }

    /// Get captured events as a Python list (parsed from the backend JSON).
    ///
    /// Returns a `list` of event dicts rather than a raw JSON string, so
    /// callers can iterate real events instead of characters.
    fn get_events(&self, py: Python<'_>) -> PyResult<PyObject> {
        let events_json = self.backend.borrow(py).get_events();
        let json_module = py.import_bound("json")?;
        let parsed = json_module.call_method1("loads", (events_json,))?;
        Ok(parsed.into())
    }

    /// Get captured events as the raw backend JSON string.
    fn get_events_json(&self, py: Python<'_>) -> String {
        self.backend.borrow(py).get_events()
    }

    /// Get statistics
    fn get_stats(&self, py: Python<'_>) -> String {
        self.backend.borrow(py).get_stats()
    }

    /// Access the underlying backend instance.
    fn backend(&self, py: Python<'_>) -> Py<Xplainit> {
        self.backend.clone_ref(py)
    }

    /// Pass-through callback for function enter
    fn on_function_enter(
        &self,
        py: Python<'_>,
        name: String,
        args: &Bound<'_, PyDict>,
        filename: String,
        line: usize,
    ) -> PyResult<()> {
        self.backend
            .borrow(py)
            .on_function_enter(name, args, filename, line)
    }

    /// Pass-through callback for function exit
    fn on_function_exit(
        &self,
        py: Python<'_>,
        name: String,
        return_value: String,
        filename: String,
        line: usize,
    ) -> PyResult<()> {
        self.backend
            .borrow(py)
            .on_function_exit(name, return_value, filename, line)
    }

    /// Pass-through callback for exceptions
    fn on_exception(
        &self,
        py: Python<'_>,
        exc_type: String,
        exc_message: String,
        filename: String,
        line: usize,
    ) -> PyResult<()> {
        self.backend
            .borrow(py)
            .on_exception(exc_type, exc_message, filename, line)
    }
}

/// Convenience function to start auto-tracing.
///
/// Enables real `sys.settrace()`-based tracing via the pure-Python
/// `xplainit.python.tracer` module and returns the backend instance that
/// receives events, so callers can retrieve them with `backend.get_events()`.
/// Call [`disable_auto_trace`] to stop.
#[pyfunction]
#[pyo3(signature = (backend=None))]
fn auto_trace(py: Python<'_>, backend: Option<&Bound<'_, PyAny>>) -> PyResult<PyObject> {
    let tracer_module = py.import_bound("xplainit.python.tracer")?;

    // Reuse a caller-provided backend, otherwise create a default Xplainit one
    // so events are actually captured (a None backend records nothing).
    let backend_obj: PyObject = match backend {
        Some(b) => b.clone().into(),
        None => Py::new(py, Xplainit::new(true, "normal", "stdout")?)?.into_py(py),
    };

    let kwargs = PyDict::new_bound(py);
    kwargs.set_item("backend", &backend_obj)?;
    // enable_tracing installs sys.settrace() and stores a module-global tracer
    // that disable_auto_trace()/disable_tracing() can later tear down.
    tracer_module.call_method("enable_tracing", (), Some(&kwargs))?;

    Ok(backend_obj)
}

/// Convenience function to stop auto-tracing started via [`auto_trace`].
#[pyfunction]
fn disable_auto_trace(py: Python<'_>) -> PyResult<()> {
    let tracer_module = py.import_bound("xplainit.python.tracer")?;
    tracer_module.call_method0("disable_tracing")?;
    Ok(())
}

// ===== Helper Functions =====

fn create_config(verbosity: &str, output: &str) -> Config {
    let verb = match verbosity.to_lowercase().as_str() {
        "brief" => Verbosity::Brief,
        "normal" => Verbosity::Normal,
        "detailed" => Verbosity::Detailed,
        "debug" => Verbosity::Debug,
        _ => Verbosity::Normal,
    };

    let output_dest = match output.to_lowercase().as_str() {
        "stdout" => OutputDestination::Stdout,
        "stderr" => OutputDestination::Stderr,
        path => OutputDestination::File(std::path::PathBuf::from(path)),
    };

    Config::new(Language::Python)
        .with_verbosity(verb)
        .with_output_destination(output_dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = create_config("normal", "stdout");
        assert_eq!(config.language, Language::Python);
    }
}
