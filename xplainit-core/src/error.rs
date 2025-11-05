use thiserror::Error;

/// Main error type for Xplainit
#[derive(Error, Debug, Clone)]
pub enum XplainitError {
    /// Parse error occurred
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// Analysis error occurred
    #[error("Analysis error: {0}")]
    AnalysisError(String),
    
    /// Unsupported language
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(String),
    
    /// Event capture error
    #[error("Event capture error: {0}")]
    EventCaptureError(String),
    
    /// Explanation generation error
    #[error("Explanation generation error: {0}")]
    ExplanationError(String),
    
    /// Internal framework error (should never happen)
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type alias for Xplainit operations
pub type Result<T> = std::result::Result<T, XplainitError>;

impl From<std::io::Error> for XplainitError {
    fn from(err: std::io::Error) -> Self {
        XplainitError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for XplainitError {
    fn from(err: serde_json::Error) -> Self {
        XplainitError::ParseError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = XplainitError::ParseError("test error".to_string());
        assert!(err.to_string().contains("Parse error"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let xplainit_err: XplainitError = io_err.into();
        assert!(matches!(xplainit_err, XplainitError::IoError(_)));
    }
}
