use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Supported programming languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Python,
    JavaScript,
    C,
    Cpp,
    Java,
    Go,
    Rust,
}

impl Language {
    /// Get language name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Python => "python",
            Language::JavaScript => "javascript",
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Java => "java",
            Language::Go => "go",
            Language::Rust => "rust",
        }
    }

    /// Parse language from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "python" | "py" => Some(Language::Python),
            "javascript" | "js" | "node" => Some(Language::JavaScript),
            "c" => Some(Language::C),
            "cpp" | "c++" | "cxx" => Some(Language::Cpp),
            "java" => Some(Language::Java),
            "go" | "golang" => Some(Language::Go),
            "rust" | "rs" => Some(Language::Rust),
            _ => None,
        }
    }
}

/// Verbosity level for explanations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Verbosity {
    /// Minimal explanations (single line)
    Brief,
    /// Standard explanations (default)
    #[default]
    Normal,
    /// Comprehensive explanations with context
    Detailed,
    /// Include framework internal details
    Debug,
}

/// Output format for explanations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OutputFormat {
    /// Plain text to console
    Console,
    /// Colored console output
    #[default]
    ConsoleColored,
    /// JSON format
    Json,
    /// HTML format
    Html,
    /// Markdown format
    Markdown,
}

/// Output destination
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OutputDestination {
    /// Standard output
    #[default]
    Stdout,
    /// Standard error
    Stderr,
    /// File
    File(PathBuf),
    /// Network endpoint
    Network(String),
    /// Multiple destinations
    Multiple(Vec<OutputDestination>),
}

/// Output mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OutputMode {
    /// Output in real-time as events occur
    #[default]
    Streaming,
    /// Buffer all output and flush at end
    Buffered,
    /// Manual control - output only when explicitly flushed
    Manual,
}

/// Main configuration for Xplainit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Programming language being traced
    pub language: Language,

    /// Verbosity level
    pub verbosity: Verbosity,

    /// Output format
    pub output_format: OutputFormat,

    /// Output destination
    pub output_destination: OutputDestination,

    /// Output mode
    pub output_mode: OutputMode,

    /// Show line numbers in output
    pub show_line_numbers: bool,

    /// Show source code snippets
    pub show_source_code: bool,

    /// Use colored output
    pub color_output: bool,

    /// Maximum recursion depth to trace
    pub max_depth: usize,

    /// Track variable values
    pub track_variables: bool,

    /// Track function calls
    pub track_function_calls: bool,

    /// Track control flow (if/else, loops)
    pub track_control_flow: bool,

    /// Capture and explain errors
    pub capture_errors: bool,

    /// Include timestamps in output
    pub include_timestamps: bool,

    /// Include thread/task IDs
    pub include_thread_info: bool,

    /// Filter: only trace these functions (empty = trace all)
    pub include_functions: Vec<String>,

    /// Filter: exclude these functions
    pub exclude_functions: Vec<String>,

    /// Filter: only trace these modules
    pub include_modules: Vec<String>,

    /// Filter: exclude these modules (e.g., standard library)
    pub exclude_modules: Vec<String>,

    /// Redact captured values whose key looks like a secret (default: true).
    ///
    /// When enabled, values keyed by names matching [`Self::redact_key_patterns`]
    /// are replaced with a redacted placeholder before they leave the process
    /// (console output, JSON serialization, dashboard payloads).
    pub redact_secrets: bool,

    /// Case-insensitive substrings that mark a value key as secret-like.
    ///
    /// Defaults to common secret names (password, token, api_key, ...). Matched
    /// as substrings so keys like `db_password` are covered.
    pub redact_key_patterns: Vec<String>,

    /// Number of *consecutive* framework errors tolerated before the runtime
    /// circuit-breaker trips and auto-disables tracing (Task 4.2 error
    /// recovery). A value of 0 disables the breaker entirely.
    ///
    /// Defaults to a small number so a persistently misbehaving trace target
    /// disables the framework quickly instead of degrading the host program.
    pub max_consecutive_errors: u64,

    /// When true, framework-internal errors are logged to stderr with context
    /// (see [`crate::control::RuntimeControl`]). When false, framework errors
    /// are counted for telemetry and then swallowed so they never reach the
    /// host program. Also enabled at runtime via the `XPLAINIT_DEBUG` env var.
    pub debug_mode: bool,
}

/// Build the default set of redaction key patterns.
fn default_redact_key_patterns() -> Vec<String> {
    crate::security::DEFAULT_REDACTION_PATTERNS
        .iter()
        .map(|s| s.to_string())
        .collect()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Language::Python,
            verbosity: Verbosity::default(),
            output_format: OutputFormat::default(),
            output_destination: OutputDestination::default(),
            output_mode: OutputMode::default(),
            show_line_numbers: true,
            show_source_code: true,
            color_output: true,
            max_depth: 100,
            track_variables: true,
            track_function_calls: true,
            track_control_flow: true,
            capture_errors: true,
            include_timestamps: false,
            include_thread_info: false,
            include_functions: Vec::new(),
            exclude_functions: Vec::new(),
            include_modules: Vec::new(),
            exclude_modules: Vec::new(),
            redact_secrets: true,
            redact_key_patterns: default_redact_key_patterns(),
            max_consecutive_errors: 5,
            debug_mode: false,
        }
    }
}

impl Config {
    /// Create new config for a specific language
    pub fn new(language: Language) -> Self {
        Self {
            language,
            ..Default::default()
        }
    }

    /// Builder method: set verbosity
    pub fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.verbosity = verbosity;
        self
    }

    /// Builder method: set output format
    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    /// Builder method: set output destination
    pub fn with_output_destination(mut self, dest: OutputDestination) -> Self {
        self.output_destination = dest;
        self
    }

    /// Builder method: set output mode
    pub fn with_output_mode(mut self, mode: OutputMode) -> Self {
        self.output_mode = mode;
        self
    }

    /// Builder method: set max depth
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Load config from environment variables
    pub fn from_env() -> Self {
        let mut config = Config::default();

        // Read XPLAINIT_* environment variables
        if let Ok(enabled) = std::env::var("XPLAINIT_ENABLED") {
            if enabled.to_lowercase() == "false" || enabled == "0" {
                // Config loaded but disabled - handled by caller
            }
        }

        if let Ok(lang) = std::env::var("XPLAINIT_LANGUAGE") {
            if let Some(language) = Language::parse(&lang) {
                config.language = language;
            }
        }

        if let Ok(verb) = std::env::var("XPLAINIT_VERBOSITY") {
            config.verbosity = match verb.to_lowercase().as_str() {
                "brief" => Verbosity::Brief,
                "normal" => Verbosity::Normal,
                "detailed" => Verbosity::Detailed,
                "debug" => Verbosity::Debug,
                _ => Verbosity::Normal,
            };
        }

        if let Ok(output) = std::env::var("XPLAINIT_OUTPUT") {
            config.output_destination = match output.to_lowercase().as_str() {
                "stdout" => OutputDestination::Stdout,
                "stderr" => OutputDestination::Stderr,
                _ => OutputDestination::File(PathBuf::from(output)),
            };
        }

        // Framework-internal debug logging (Task 4.2). When set to a truthy
        // value, framework errors are logged to stderr instead of silently
        // swallowed.
        if let Ok(debug) = std::env::var("XPLAINIT_DEBUG") {
            let lower = debug.to_lowercase();
            config.debug_mode = lower != "false" && debug != "0" && !debug.is_empty();
        }

        if let Ok(max_errors) = std::env::var("XPLAINIT_MAX_CONSECUTIVE_ERRORS") {
            if let Ok(parsed) = max_errors.parse::<u64>() {
                config.max_consecutive_errors = parsed;
            }
        }

        // Secret redaction toggle (Task 4.1). Defaults to on; only an explicit
        // falsey value disables it so redaction can never be turned off by
        // accident on the load paths that serialize events out of the process.
        if let Ok(redact) = std::env::var("XPLAINIT_REDACT_SECRETS") {
            let lower = redact.to_lowercase();
            config.redact_secrets = lower != "false" && redact != "0";
        }

        // Custom redaction key patterns. A comma-separated list *replaces* the
        // default pattern set so a caller can narrow or extend which value keys
        // are treated as secret-like. Empty entries are ignored; an all-empty
        // value leaves the defaults in place rather than disabling redaction.
        if let Ok(patterns) = std::env::var("XPLAINIT_REDACT_KEY_PATTERNS") {
            let parsed: Vec<String> = patterns
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !parsed.is_empty() {
                config.redact_key_patterns = parsed;
            }
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_str() {
        assert_eq!(Language::parse("python"), Some(Language::Python));
        assert_eq!(Language::parse("py"), Some(Language::Python));
        assert_eq!(Language::parse("javascript"), Some(Language::JavaScript));
        assert_eq!(Language::parse("js"), Some(Language::JavaScript));
        assert_eq!(Language::parse("invalid"), None);
    }

    #[test]
    fn test_config_builder() {
        let config = Config::new(Language::Python)
            .with_verbosity(Verbosity::Detailed)
            .with_max_depth(50);

        assert_eq!(config.language, Language::Python);
        assert_eq!(config.verbosity, Verbosity::Detailed);
        assert_eq!(config.max_depth, 50);
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.verbosity, Verbosity::Normal);
        assert_eq!(config.max_depth, 100);
        assert!(config.track_variables);
        assert!(config.capture_errors);
    }

    #[test]
    fn test_redaction_defaults_on_with_secret_patterns() {
        let config = Config::default();
        // Redaction must default to on (safe-by-default policy).
        assert!(config.redact_secrets);
        // The default pattern set must include the common secret names.
        for expected in ["password", "token", "api_key", "secret", "private_key"] {
            assert!(
                config.redact_key_patterns.iter().any(|p| p == expected),
                "default redaction patterns missing '{expected}'"
            );
        }
    }

    #[test]
    fn test_from_env_parses_custom_redact_key_patterns() {
        // Env vars are process-global; set, read, and immediately restore so
        // this test does not pollute others (which is why it does its own
        // save/restore rather than relying on external isolation).
        let key = "XPLAINIT_REDACT_KEY_PATTERNS";
        let prev = std::env::var(key).ok();
        std::env::set_var(key, "cookie, ssn");
        let config = Config::from_env();
        match prev {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }

        // Custom list replaces the defaults, so the env-provided patterns are
        // present and the built-in "password" default is gone. Fails if
        // from_env does not parse XPLAINIT_REDACT_KEY_PATTERNS.
        assert!(config.redact_key_patterns.iter().any(|p| p == "cookie"));
        assert!(config.redact_key_patterns.iter().any(|p| p == "ssn"));
        assert!(
            !config.redact_key_patterns.iter().any(|p| p == "password"),
            "custom patterns should replace defaults"
        );
    }

    #[test]
    fn test_from_env_parses_redact_secrets_toggle() {
        let key = "XPLAINIT_REDACT_SECRETS";
        let prev = std::env::var(key).ok();
        std::env::set_var(key, "false");
        let config = Config::from_env();
        match prev {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
        assert!(
            !config.redact_secrets,
            "XPLAINIT_REDACT_SECRETS=false should disable redaction"
        );
    }
}
