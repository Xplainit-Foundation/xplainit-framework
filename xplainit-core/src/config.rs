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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
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
}
