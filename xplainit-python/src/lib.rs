/// Python bindings for Xplainit Framework
/// Provides sys.settrace() integration for runtime code explanation

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString, PyTuple};
use xplainit_core::*;
use std::sync::Arc;
use parking_lot::RwLock;

mod tracer;
mod decorators;

use tracer::PythonTracer;

/// Python module initialization
#[pymodule]
fn xplainit(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Xplainit>()?;
    m.add_class::<XplainitContext>()?;
    m.add_function(wrap_pyfunction!(enable, m)?)?;
    m.add_function(wrap_pyfunction!(disable, m)?)?;
    m.add_function(wrap_pyfunction!(is_enabled, m)?)?;
    m.add_function(wrap_pyfunction!(explain_function, m)?)?;
    m.add_function(wrap_pyfunction!(get_last_explanation, m)?)?;
    
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
    
    /// Start tracing (installs sys.settrace)
    fn start(&self, py: Python) -> PyResult<()> {
        self.tracer.write().start(py)
    }
    
    /// Stop tracing (removes sys.settrace)
    fn stop(&self, py: Python) -> PyResult<()> {
        self.tracer.write().stop(py)
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
}

/// Context manager for scoped tracing
#[pyclass]
struct XplainitContext {
    tracer: Arc<RwLock<PythonTracer>>,
    was_enabled: bool,
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
            was_enabled: false,
        }
    }
    
    fn __enter__(&mut self, py: Python) -> PyResult<()> {
        self.was_enabled = self.tracer.read().is_enabled();
        self.tracer.write().start(py)?;
        Ok(())
    }
    
    fn __exit__(
        &mut self,
        py: Python,
        _exc_type: Option<&PyAny>,
        _exc_value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> PyResult<bool> {
        self.tracer.write().stop(py)?;
        if !self.was_enabled {
            self.tracer.write().disable();
        }
        Ok(false) // Don't suppress exceptions
    }
    
    fn get_events(&self) -> String {
        self.tracer.read().get_events_json()
    }
}

// ===== Module-level functions =====

/// Enable global tracing
#[pyfunction]
fn enable(py: Python) -> PyResult<()> {
    let mut global = GLOBAL_INSTANCE.write();
    if global.is_none() {
        let config = create_config("normal", "stdout");
        *global = Some(PythonTracer::new(config, true));
    }
    
    if let Some(tracer) = global.as_mut() {
        tracer.enable();
        tracer.start(py)?;
    }
    
    Ok(())
}

/// Disable global tracing
#[pyfunction]
fn disable(py: Python) -> PyResult<()> {
    if let Some(tracer) = GLOBAL_INSTANCE.write().as_mut() {
        tracer.stop(py)?;
        tracer.disable();
    }
    Ok(())
}

/// Check if global tracing is enabled
#[pyfunction]
fn is_enabled() -> bool {
    GLOBAL_INSTANCE
        .read()
        .as_ref()
        .map(|t| t.is_enabled())
        .unwrap_or(false)
}

/// Decorator function to explain a specific function
#[pyfunction]
fn explain_function(func: &PyAny) -> PyResult<PyObject> {
    decorators::create_explain_decorator(func)
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
