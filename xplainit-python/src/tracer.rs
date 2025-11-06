//! Python tracer implementation using sys.settrace()

use pyo3::prelude::*;
use xplainit_core::*;

#[allow(dead_code)]
pub struct PythonTracer {
    config: Config,
    runtime: RuntimeEngine,
    control: RuntimeControl,
    explainer: ExplanationGenerator,
    enabled: bool,
    last_explanation: String,
    event_count: usize,
}

impl PythonTracer {
    pub fn new(config: Config, enabled: bool) -> Self {
        let runtime = RuntimeEngine::new(config.clone());
        let control = RuntimeControl::new(config.clone());
        let explainer = ExplanationGenerator::new(VerbosityLevel::Normal);
        
        Self {
            config,
            runtime,
            control,
            explainer,
            enabled,
            last_explanation: String::new(),
            event_count: 0,
        }
    }
    
    pub fn enable(&mut self) {
        self.enabled = true;
        self.control.enable();
    }
    
    pub fn disable(&mut self) {
        self.enabled = false;
        self.control.disable();
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled && self.control.is_enabled()
    }
    
    #[allow(dead_code)]
    pub fn start(&mut self, py: Python) -> PyResult<()> {
        if !self.is_enabled() {
            return Ok(());
        }
        
        // Install trace function
        let sys = py.import_bound("sys")?;
        let trace_func = create_trace_function()?;
        sys.setattr("settrace", trace_func)?;
        
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn stop(&mut self, py: Python) -> PyResult<()> {
        let sys = py.import_bound("sys")?;
        sys.setattr("settrace", py.None())?;
        Ok(())
    }
    
    pub fn clear(&mut self) {
        self.event_count = 0;
        self.last_explanation.clear();
    }
    
    pub fn set_verbosity(&mut self, level: &str) {
        let verb = match level.to_lowercase().as_str() {
            "brief" => VerbosityLevel::Brief,
            "normal" => VerbosityLevel::Normal,
            "detailed" => VerbosityLevel::Detailed,
            "debug" => VerbosityLevel::Debug,
            _ => VerbosityLevel::Normal,
        };
        self.explainer = ExplanationGenerator::new(verb);
    }
    
    pub fn get_events_json(&self) -> String {
        // Get events from runtime engine
        let events = self.runtime.get_events();
        serde_json::to_string_pretty(&events).unwrap_or_else(|_| "[]".to_string())
    }
    
    pub fn get_last_explanation(&self) -> String {
        self.last_explanation.clone()
    }
    
    pub fn get_stats(&self) -> String {
        format!("Events captured: {}, Enabled: {}", self.event_count, self.is_enabled())
    }
    
    /// Record a simple event (for basic demonstration)
    /// Note: This is a simplified version. Full sys.settrace() integration TODO.
    #[allow(dead_code)]
    pub fn record_event(&mut self, event: ExecutionEvent) -> PyResult<()> {
        if !self.control.should_capture_event() {
            return Ok(());
        }
        
        // TODO: Add event to runtime engine when API is available
        // For now, just generate explanation directly
        
        // Generate explanation
        if self.control.is_explain_enabled() {
            self.last_explanation = self.explainer.explain(&event);
            
            // Print to stdout/stderr based on config
            if event.is_error() {
                eprintln!("{}", self.last_explanation);
            } else {
                println!("{}", self.last_explanation);
            }
        }
        
        self.event_count += 1;
        Ok(())
    }
}

// ===== Helper Functions =====

#[allow(dead_code)]
fn create_trace_function() -> PyResult<PyObject> {
    Python::with_gil(|py| {
        // This will be the actual trace function
        // For now, return a placeholder - full implementation needed
        Ok(py.None())
    })
}

#[allow(dead_code)]
fn python_to_value(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    if obj.is_none() {
        return Ok(Value::Null);
    }
    
    if let Ok(b) = obj.extract::<bool>() {
        return Ok(Value::Bool(b));
    }
    
    if let Ok(i) = obj.extract::<i64>() {
        return Ok(Value::Integer(i));
    }
    
    if let Ok(f) = obj.extract::<f64>() {
        return Ok(Value::Float(f));
    }
    
    if let Ok(s) = obj.extract::<String>() {
        return Ok(Value::String(s));
    }
    
    if obj.is_callable() {
        if let Ok(name) = obj.getattr("__name__") {
            if let Ok(n) = name.extract::<String>() {
                return Ok(Value::Function(n));
            }
        }
        return Ok(Value::Function("<lambda>".to_string()));
    }
    
    // Default: try to convert to string representation
    if let Ok(repr) = obj.str() {
        if let Ok(s) = repr.extract::<String>() {
            return Ok(Value::Unknown(s));
        }
    }
    
    Ok(Value::Unknown("<object>".to_string()))
}
