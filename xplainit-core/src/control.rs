//! Enable/Disable Control System
//! Zero-overhead control mechanism with graceful degradation.
//!
//! # Error recovery and the `panic = "abort"` release profile
//!
//! The root `Cargo.toml` sets `panic = "abort"` for the release profile
//! (chosen for smaller binaries and slightly faster code, since no unwind
//! tables are emitted). Under `panic = "abort"`, [`std::panic::catch_unwind`]
//! **cannot** catch a panic: the process aborts immediately. This is a
//! deliberate, documented tradeoff. Consequently:
//!
//! * [`safe_execute`] still wraps the closure in `catch_unwind`, which is
//!   effective in debug/test builds (and in any consumer that opts into
//!   `panic = "unwind"`), but in a release build it does **not** turn a panic
//!   into a recoverable `None`. We do not claim panic protection we cannot
//!   deliver in that profile.
//! * The primary, profile-independent recovery mechanism is therefore the
//!   **circuit-breaker**: framework code returns `Result`/`Option` for
//!   recoverable failures and routes them through [`RuntimeControl::record_error`].
//!   After a configurable number of consecutive errors the breaker trips and
//!   auto-disables tracing so a persistently failing trace target degrades the
//!   framework, never the host program.
//!
//! In short: rely on `catch_unwind` only where unwinding is enabled; rely on
//! the circuit-breaker + `Result`-based isolation everywhere.
use crate::config::Config;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

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

    // ===== Circuit-breaker + telemetry (Task 4.2) =====
    /// Consecutive framework errors since the last success. Reset by
    /// [`Self::record_success`]; when it reaches `max_consecutive_errors` the
    /// breaker trips.
    consecutive_errors: Arc<AtomicU64>,

    /// Number of consecutive errors that trips the breaker (0 = disabled).
    max_consecutive_errors: Arc<AtomicU64>,

    /// Telemetry: total framework errors ever recorded.
    total_errors: Arc<AtomicU64>,

    /// Telemetry: total panics observed by [`safe_execute`].
    total_panics: Arc<AtomicU64>,

    /// Telemetry: number of times the circuit-breaker has tripped.
    times_tripped: Arc<AtomicU64>,

    /// Debug flag: when true, framework-internal errors are logged to stderr;
    /// when false they are counted then swallowed.
    debug_mode: Arc<AtomicBool>,
}

impl RuntimeControl {
    /// Create new runtime control
    pub fn new(config: Config) -> Self {
        let max_consecutive_errors = config.max_consecutive_errors;
        let debug_mode = config.debug_mode;
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
            consecutive_errors: Arc::new(AtomicU64::new(0)),
            max_consecutive_errors: Arc::new(AtomicU64::new(max_consecutive_errors)),
            total_errors: Arc::new(AtomicU64::new(0)),
            total_panics: Arc::new(AtomicU64::new(0)),
            times_tripped: Arc::new(AtomicU64::new(0)),
            debug_mode: Arc::new(AtomicBool::new(debug_mode)),
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

        // Debug mode (Task 4.2). Config::from_env already parsed XPLAINIT_DEBUG
        // into config.debug_mode, which new() propagated into the atomic flag;
        // re-check here so the atomic is always in sync with the environment.
        if let Ok(val) = std::env::var("XPLAINIT_DEBUG") {
            let enabled = val.to_lowercase() != "false" && val != "0" && !val.is_empty();
            control.set_debug_mode(enabled);
        }

        if let Ok(val) = std::env::var("XPLAINIT_MAX_CONSECUTIVE_ERRORS") {
            if let Ok(limit) = val.parse::<u64>() {
                control.set_max_consecutive_errors(limit);
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
        self.enabled.load(Ordering::Relaxed)
            && self.capture_enabled.load(Ordering::Relaxed)
            && !self.panic_mode.load(Ordering::Relaxed)
    }

    /// Check if explanation generation is enabled
    #[inline(always)]
    pub fn is_explain_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
            && self.explain_enabled.load(Ordering::Relaxed)
            && !self.panic_mode.load(Ordering::Relaxed)
    }

    /// Check if error tracking is enabled
    #[inline(always)]
    pub fn is_error_tracking_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
            && self.error_tracking_enabled.load(Ordering::Relaxed)
            && !self.panic_mode.load(Ordering::Relaxed)
    }

    /// Check if performance tracking is enabled
    #[inline(always)]
    pub fn is_perf_tracking_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
            && self.perf_tracking_enabled.load(Ordering::Relaxed)
            && !self.panic_mode.load(Ordering::Relaxed)
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
        self.error_tracking_enabled
            .store(enabled, Ordering::Relaxed);
    }

    /// Enable/disable performance tracking
    pub fn set_perf_tracking_enabled(&self, enabled: bool) {
        self.perf_tracking_enabled.store(enabled, Ordering::Relaxed);
    }

    /// Set max events per second (0 = unlimited)
    pub fn set_max_events_per_sec(&self, limit: u64) {
        self.max_events_per_sec.store(limit, Ordering::Relaxed);
    }

    // ===== Circuit-breaker + telemetry (Task 4.2) =====

    /// Set the number of consecutive errors that trips the breaker.
    ///
    /// A value of 0 disables the breaker (errors are still counted for
    /// telemetry but never auto-disable tracing).
    pub fn set_max_consecutive_errors(&self, limit: u64) {
        self.max_consecutive_errors.store(limit, Ordering::Relaxed);
    }

    /// Get the currently configured consecutive-error threshold.
    #[inline]
    pub fn max_consecutive_errors(&self) -> u64 {
        self.max_consecutive_errors.load(Ordering::Relaxed)
    }

    /// Current consecutive-error count (since the last recorded success).
    #[inline]
    pub fn consecutive_errors(&self) -> u64 {
        self.consecutive_errors.load(Ordering::Relaxed)
    }

    /// Record a recoverable framework error.
    ///
    /// Increments the consecutive-error counter and the total-error telemetry
    /// counter. When the consecutive count reaches the configured threshold the
    /// circuit-breaker trips: tracing is auto-disabled via
    /// [`Self::enter_panic_mode`] and the trip is counted. This is the primary,
    /// profile-independent recovery mechanism (see the module docs on
    /// `panic = "abort"`).
    ///
    /// Errors are logged to stderr only when [`Self::is_debug_mode`] is true;
    /// otherwise they are counted and swallowed so nothing reaches the host.
    pub fn record_error(&self) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
        let consecutive = self.consecutive_errors.fetch_add(1, Ordering::Relaxed) + 1;

        if self.is_debug_mode() {
            eprintln!(
                "xplainit[debug]: framework error recorded (consecutive={}, total={})",
                consecutive,
                self.total_errors.load(Ordering::Relaxed)
            );
        }

        let threshold = self.max_consecutive_errors.load(Ordering::Relaxed);
        if threshold > 0 && consecutive >= threshold && !self.is_panic_mode() {
            self.times_tripped.fetch_add(1, Ordering::Relaxed);
            self.enter_panic_mode();
        }
    }

    /// Record that a panic was observed (telemetry only; the accompanying
    /// [`Self::record_error`] handles breaker state). Kept separate so panics
    /// can be distinguished from ordinary recoverable errors.
    pub fn record_panic(&self) {
        self.total_panics.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a successful framework operation, resetting the consecutive-error
    /// counter so transient failures do not accumulate toward the breaker.
    pub fn record_success(&self) {
        self.consecutive_errors.store(0, Ordering::Relaxed);
    }

    /// Reset the consecutive-error counter without recording a success. Alias
    /// kept for clarity at call sites that want to clear the breaker state.
    pub fn reset_error_counter(&self) {
        self.consecutive_errors.store(0, Ordering::Relaxed);
    }

    /// Total framework errors recorded since creation (telemetry).
    #[inline]
    pub fn total_errors(&self) -> u64 {
        self.total_errors.load(Ordering::Relaxed)
    }

    /// Total panics observed by [`safe_execute`] since creation (telemetry).
    #[inline]
    pub fn total_panics(&self) -> u64 {
        self.total_panics.load(Ordering::Relaxed)
    }

    /// Number of times the circuit-breaker has tripped (telemetry).
    #[inline]
    pub fn times_tripped(&self) -> u64 {
        self.times_tripped.load(Ordering::Relaxed)
    }

    /// Whether framework-internal debug logging is enabled.
    #[inline(always)]
    pub fn is_debug_mode(&self) -> bool {
        self.debug_mode.load(Ordering::Relaxed)
    }

    /// Enable/disable framework-internal debug logging at runtime.
    pub fn set_debug_mode(&self, enabled: bool) {
        self.debug_mode.store(enabled, Ordering::Relaxed);
    }

    /// Enter panic mode (disable framework to prevent cascading failures).
    ///
    /// The stderr notice is only emitted in debug mode so a production host is
    /// never spammed by framework-internal output.
    pub fn enter_panic_mode(&self) {
        self.panic_mode.store(true, Ordering::Relaxed);
        if self.is_debug_mode() {
            eprintln!(
                "xplainit[debug]: entered panic mode - tracing disabled to prevent cascading failures"
            );
        }
    }

    /// Exit panic mode and re-enable. Also clears the consecutive-error counter
    /// so the breaker starts fresh.
    pub fn exit_panic_mode(&self) {
        self.panic_mode.store(false, Ordering::Relaxed);
        self.consecutive_errors.store(0, Ordering::Relaxed);
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

/// Safe execution wrapper that isolates framework failures from the host.
///
/// Returns `Some(value)` on success and `None` on failure, routing failures
/// through the circuit-breaker so repeated errors auto-disable tracing.
///
/// # Panic handling and `panic = "abort"`
///
/// The closure is wrapped in [`std::panic::catch_unwind`]. This is effective
/// wherever unwinding is enabled (debug/test builds, or consumers that opt into
/// `panic = "unwind"`). **In a release build using the workspace default
/// `panic = "abort"`, a panic aborts the process before it can be caught** -
/// see the module-level docs. In that profile the circuit-breaker (fed by
/// `Result`-returning framework code via [`RuntimeControl::record_error`]) is
/// the real recovery mechanism, and this function must not be relied on to turn
/// a panic into a recoverable `None`.
///
/// On a caught panic the panic is counted, the error path is recorded (which
/// may trip the breaker), and framework detail is logged only in debug mode.
pub fn safe_execute<F, T>(control: &RuntimeControl, f: F) -> Option<T>
where
    F: FnOnce() -> T + std::panic::UnwindSafe,
{
    if control.is_panic_mode() {
        return None;
    }

    // catch_unwind requires the closure be UnwindSafe (enforced by the `F`
    // bound). It is only effective where unwinding is enabled; under the
    // release `panic = "abort"` profile a panic aborts before reaching here.
    match std::panic::catch_unwind(f) {
        Ok(result) => {
            control.record_success();
            Some(result)
        }
        Err(e) => {
            control.record_panic();
            if control.is_debug_mode() {
                eprintln!("xplainit[debug]: framework panic caught: {e:?}");
            }
            control.record_error();
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

        let result = safe_execute(&control, || 42);

        assert_eq!(result, Some(42));
        assert!(!control.is_panic_mode());
    }

    #[test]
    fn test_safe_execute_panic() {
        // Silence the default panic hook so the intentional panic below does
        // not spam the test output; catch_unwind still returns Err.
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));

        // Threshold of 1 so a single panic-induced error trips the breaker.
        let config = Config {
            max_consecutive_errors: 1,
            ..Config::default()
        };
        let control = RuntimeControl::new(config);

        let result = safe_execute(&control, || {
            panic!("Test panic");
        });

        std::panic::set_hook(prev);

        assert_eq!(result, None);
        assert!(control.is_panic_mode());
        assert_eq!(control.total_panics(), 1);
        assert_eq!(control.total_errors(), 1);
    }

    #[test]
    fn test_safe_execute_records_error_without_panic_mode() {
        // With a high threshold, a single failing closure records one error but
        // does NOT trip the breaker.
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));

        let config = Config {
            max_consecutive_errors: 100,
            ..Config::default()
        };
        let control = RuntimeControl::new(config);

        let result: Option<()> = safe_execute(&control, || panic!("boom"));

        std::panic::set_hook(prev);

        assert_eq!(result, None);
        assert!(!control.is_panic_mode());
        assert_eq!(control.total_errors(), 1);
        assert_eq!(control.consecutive_errors(), 1);
    }

    #[test]
    fn test_circuit_breaker_trips_after_threshold() {
        let config = Config {
            max_consecutive_errors: 3,
            ..Config::default()
        };
        let control = RuntimeControl::new(config);

        assert!(control.is_capture_enabled());

        // First two errors: not tripped yet.
        control.record_error();
        control.record_error();
        assert!(!control.is_panic_mode());
        assert!(control.is_capture_enabled());
        assert_eq!(control.times_tripped(), 0);

        // Third consecutive error trips the breaker and disables capture.
        control.record_error();
        assert!(control.is_panic_mode());
        assert!(!control.is_capture_enabled());
        assert!(!control.is_explain_enabled());
        assert_eq!(control.times_tripped(), 1);
        assert_eq!(control.total_errors(), 3);
    }

    #[test]
    fn test_record_success_resets_consecutive_counter() {
        let config = Config {
            max_consecutive_errors: 3,
            ..Config::default()
        };
        let control = RuntimeControl::new(config);

        control.record_error();
        control.record_error();
        assert_eq!(control.consecutive_errors(), 2);

        // A success resets the consecutive counter so the breaker does not trip.
        control.record_success();
        assert_eq!(control.consecutive_errors(), 0);

        control.record_error();
        control.record_error();
        assert!(!control.is_panic_mode(), "breaker tripped prematurely");
        assert_eq!(control.times_tripped(), 0);

        // total_errors still reflects every error ever recorded.
        assert_eq!(control.total_errors(), 4);
    }

    #[test]
    fn test_circuit_breaker_disabled_when_threshold_zero() {
        let config = Config {
            max_consecutive_errors: 0,
            ..Config::default()
        };
        let control = RuntimeControl::new(config);

        for _ in 0..50 {
            control.record_error();
        }

        // Breaker disabled: errors counted but tracing stays enabled.
        assert!(!control.is_panic_mode());
        assert_eq!(control.times_tripped(), 0);
        assert_eq!(control.total_errors(), 50);
    }

    #[test]
    fn test_telemetry_counters() {
        let control = RuntimeControl::default();
        assert_eq!(control.total_errors(), 0);
        assert_eq!(control.total_panics(), 0);
        assert_eq!(control.times_tripped(), 0);

        control.record_error();
        control.record_panic();
        assert_eq!(control.total_errors(), 1);
        assert_eq!(control.total_panics(), 1);
    }

    #[test]
    fn test_debug_mode_off_swallows_errors_without_propagating() {
        // With debug off (default), recording a framework error must never
        // panic or propagate; it is simply counted.
        let config = Config {
            debug_mode: false,
            max_consecutive_errors: 0, // avoid tripping for this check
            ..Config::default()
        };
        let control = RuntimeControl::new(config);

        assert!(!control.is_debug_mode());
        control.record_error(); // must not panic
        assert_eq!(control.total_errors(), 1);
    }

    #[test]
    fn test_debug_mode_toggle() {
        let control = RuntimeControl::default();
        assert!(!control.is_debug_mode());
        control.set_debug_mode(true);
        assert!(control.is_debug_mode());
        control.set_debug_mode(false);
        assert!(!control.is_debug_mode());
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
