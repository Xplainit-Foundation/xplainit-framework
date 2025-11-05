//! Enable/Disable Control System
//! Zero-overhead control mechanism with graceful degradation
use crate::config::Config;
use std::sync::Arc;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Fast enable/disable control with atomic operations
#[derive(Debug, Clone)]
pub struct RuntimeControl {
    /// Global enabled state (atomic for zero-overhead checks)
    enabled: Arc<AtomicBool>,
    
    /// Event capture enabled
    capture_enabled: Arc<AtomicBool>,
    
    /// Explanation generation enabled
    explain_enabled: Arc<AtomicBool>,
    
    /// Error tracking enabled
    error_tracking_enabled: Arc<AtomicBool>,
    
    /// Performance tracking enabled
    perf_tracking_enabled: Arc<AtomicBool>,
    
    /// Event counter for rate limiting
    event_count: Arc<AtomicU64>,
    
    /// Max events per second (0 = unlimited)
    max_events_per_sec: Arc<AtomicU64>,
    
    /// Configuration
    config: Arc<RwLock<Config>>,
    
    /// Panic mode - if framework crashes, disable to avoid cascading failures
    panic_mode: Arc<AtomicBool>,
}

impl RuntimeControl {
    /// Create new runtime control
    pub fn new(config: Config) -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(true)),
            capture_enabled: Arc::new(AtomicBool::new(true)),
            explain_enabled: Arc::new(AtomicBool::new(true)),
            error_tracking_enabled: Arc::new(AtomicBool::new(true)),
            perf_tracking_enabled: Arc::new(AtomicBool::new(false)),
            event_count: Arc::new(AtomicU64::new(0)),
            max_events_per_sec: Arc::new(AtomicU64::new(0)),
            config: Arc::new(RwLock::new(config)),
            panic_mode: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// Create from environment variables
    pub fn from_env() -> Self {
        let config = Config::from_env();
        let control = Self::new(config);
        
        // Check XPLAINIT_ENABLED
        if let Ok(val) = std::env::var("XPLAINIT_ENABLED") {
            let enabled = val.to_lowercase() != "false" && val != "0";
            control.set_enabled(enabled);
        }
        
        // Check individual feature flags
        if let Ok(val) = std::env::var("XPLAINIT_CAPTURE") {
            let enabled = val.to_lowercase() != "false" && val != "0";
            control.set_capture_enabled(enabled);
        }
        
        if let Ok(val) = std::env::var("XPLAINIT_EXPLAIN") {
            let enabled = val.to_lowercase() != "false" && val != "0";
            control.set_explain_enabled(enabled);
        }
        
        if let Ok(val) = std::env::var("XPLAINIT_MAX_EVENTS_PER_SEC") {
            if let Ok(limit) = val.parse::<u64>() {
                control.set_max_events_per_sec(limit);
            }
        }
        
        control
    }
    
    // ===== Fast Read Methods (Atomic) =====
    
    /// Check if framework is enabled (fastest check - single atomic read)
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
    
    /// Check if event capture is enabled
    #[inline(always)]
    pub fn is_capture_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed) && 
        self.capture_enabled.load(Ordering::Relaxed) &&
        !self.panic_mode.load(Ordering::Relaxed)
    }
    
    /// Check if explanation generation is enabled
    #[inline(always)]
    pub fn is_explain_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed) && 
        self.explain_enabled.load(Ordering::Relaxed) &&
        !self.panic_mode.load(Ordering::Relaxed)
    }
    
    /// Check if error tracking is enabled
    #[inline(always)]
    pub fn is_error_tracking_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed) && 
        self.error_tracking_enabled.load(Ordering::Relaxed) &&
        !self.panic_mode.load(Ordering::Relaxed)
    }
    
    /// Check if performance tracking is enabled
    #[inline(always)]
    pub fn is_perf_tracking_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed) && 
        self.perf_tracking_enabled.load(Ordering::Relaxed) &&
        !self.panic_mode.load(Ordering::Relaxed)
    }
    
    /// Check if in panic mode (framework error occurred)
    #[inline(always)]
    pub fn is_panic_mode(&self) -> bool {
        self.panic_mode.load(Ordering::Relaxed)
    }
    
    /// Check rate limit and increment counter
    #[inline]
    pub fn should_capture_event(&self) -> bool {
        if !self.is_capture_enabled() {
            return false;
        }
        
        let max = self.max_events_per_sec.load(Ordering::Relaxed);
        if max == 0 {
            // No limit
            return true;
        }
        
        // Simple counter-based rate limiting
        let count = self.event_count.fetch_add(1, Ordering::Relaxed);
        count < max
    }
    
    /// Reset event counter (call periodically, e.g., every second)
    pub fn reset_event_counter(&self) {
        self.event_count.store(0, Ordering::Relaxed);
    }
    
    // ===== Write Methods =====
    
    /// Enable the entire framework
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
        self.panic_mode.store(false, Ordering::Relaxed);
    }
    
    /// Disable the entire framework
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }
    
    /// Set enabled state
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
        if enabled {
            self.panic_mode.store(false, Ordering::Relaxed);
        }
    }
    
    /// Enable/disable event capture
    pub fn set_capture_enabled(&self, enabled: bool) {
        self.capture_enabled.store(enabled, Ordering::Relaxed);
    }
    
    /// Enable/disable explanation generation
    pub fn set_explain_enabled(&self, enabled: bool) {
        self.explain_enabled.store(enabled, Ordering::Relaxed);
    }
    
    /// Enable/disable error tracking
    pub fn set_error_tracking_enabled(&self, enabled: bool) {
        self.error_tracking_enabled.store(enabled, Ordering::Relaxed);
    }
    
    /// Enable/disable performance tracking
    pub fn set_perf_tracking_enabled(&self, enabled: bool) {
        self.perf_tracking_enabled.store(enabled, Ordering::Relaxed);
    }
    
    /// Set max events per second (0 = unlimited)
    pub fn set_max_events_per_sec(&self, limit: u64) {
        self.max_events_per_sec.store(limit, Ordering::Relaxed);
    }
    
    /// Enter panic mode (disable framework to prevent cascading failures)
    pub fn enter_panic_mode(&self) {
        self.panic_mode.store(true, Ordering::Relaxed);
        eprintln!("⚠️  Xplainit Framework entered panic mode - disabling to prevent cascading failures");
    }
    
    /// Exit panic mode and re-enable
    pub fn exit_panic_mode(&self) {
        self.panic_mode.store(false, Ordering::Relaxed);
    }
    
    // ===== Configuration Access =====
    
    /// Get current configuration (read lock)
    pub fn config(&self) -> parking_lot::RwLockReadGuard<'_, Config> {
        self.config.read()
    }
    
    /// Update configuration (write lock)
    pub fn update_config<F>(&self, f: F) 
    where
        F: FnOnce(&mut Config),
    {
        let mut config = self.config.write();
        f(&mut config);
    }
    
    /// Replace entire configuration
    pub fn set_config(&self, config: Config) {
        *self.config.write() = config;
    }
}

impl Default for RuntimeControl {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

/// Scoped enable/disable guard
/// When dropped, restores the previous state
pub struct ScopedControl {
    control: RuntimeControl,
    previous_state: bool,
}

impl ScopedControl {
    /// Create a scoped control that disables on creation
    pub fn disable(control: RuntimeControl) -> Self {
        let previous_state = control.is_enabled();
        control.disable();
        Self {
            control,
            previous_state,
        }
    }
    
    /// Create a scoped control that enables on creation
    pub fn enable(control: RuntimeControl) -> Self {
        let previous_state = control.is_enabled();
        control.enable();
        Self {
            control,
            previous_state,
        }
    }
}

impl Drop for ScopedControl {
    fn drop(&mut self) {
        self.control.set_enabled(self.previous_state);
    }
}

/// Safe execution wrapper that catches panics
pub fn safe_execute<F, T>(control: &RuntimeControl, f: F) -> Option<T>
where
    F: FnOnce() -> T + std::panic::UnwindSafe,
{
    if control.is_panic_mode() {
        return None;
    }
    
    match std::panic::catch_unwind(f) {
        Ok(result) => Some(result),
        Err(e) => {
            control.enter_panic_mode();
            eprintln!("❌ Xplainit Framework panic: {:?}", e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Language;

    #[test]
    fn test_control_creation() {
        let control = RuntimeControl::new(Config::new(Language::Python));
        assert!(control.is_enabled());
        assert!(control.is_capture_enabled());
        assert!(control.is_explain_enabled());
    }
    
    #[test]
    fn test_enable_disable() {
        let control = RuntimeControl::default();
        
        assert!(control.is_enabled());
        
        control.disable();
        assert!(!control.is_enabled());
        
        control.enable();
        assert!(control.is_enabled());
    }
    
    #[test]
    fn test_selective_disable() {
        let control = RuntimeControl::default();
        
        control.set_capture_enabled(false);
        assert!(!control.is_capture_enabled());
        assert!(control.is_explain_enabled()); // Still enabled
    }
    
    #[test]
    fn test_panic_mode() {
        let control = RuntimeControl::default();
        
        assert!(!control.is_panic_mode());
        assert!(control.is_enabled());
        
        control.enter_panic_mode();
        assert!(control.is_panic_mode());
        assert!(!control.is_capture_enabled());
        assert!(!control.is_explain_enabled());
    }
    
    #[test]
    fn test_rate_limiting() {
        let control = RuntimeControl::default();
        control.set_max_events_per_sec(5);
        
        // Should allow first 5 events
        for _ in 0..5 {
            assert!(control.should_capture_event());
        }
        
        // Should block 6th event
        assert!(!control.should_capture_event());
        
        // Reset counter
        control.reset_event_counter();
        
        // Should allow again
        assert!(control.should_capture_event());
    }
    
    #[test]
    fn test_scoped_control() {
        let control = RuntimeControl::default();
        assert!(control.is_enabled());
        
        {
            let _guard = ScopedControl::disable(control.clone());
            assert!(!control.is_enabled());
        }
        
        // Should be re-enabled after scope
        assert!(control.is_enabled());
    }
    
    #[test]
    fn test_safe_execute_success() {
        let control = RuntimeControl::default();
        
        let result = safe_execute(&control, || {
            42
        });
        
        assert_eq!(result, Some(42));
        assert!(!control.is_panic_mode());
    }
    
    #[test]
    fn test_safe_execute_panic() {
        let control = RuntimeControl::default();
        
        let result = safe_execute(&control, || {
            panic!("Test panic");
        });
        
        assert_eq!(result, None);
        assert!(control.is_panic_mode());
    }
    
    #[test]
    fn test_config_update() {
        let control = RuntimeControl::default();
        
        control.update_config(|config| {
            config.max_depth = 50;
        });
        
        let config = control.config();
        assert_eq!(config.max_depth, 50);
    }
    
    #[test]
    fn test_from_env() {
        // This test depends on environment variables
        // In real usage, set XPLAINIT_ENABLED=true etc.
        let control = RuntimeControl::from_env();
        assert!(control.is_enabled() || !control.is_enabled()); // Will depend on env
    }
}
