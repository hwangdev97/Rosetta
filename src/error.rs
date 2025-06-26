use thiserror::Error;

/// Custom error types for the translator
#[derive(Error, Debug)]
pub enum TranslatorError {
    #[error("API request failed with status {status}: {message}")]
    ApiError { status: u16, message: String },

    #[error("Translation failed: {0}")]
    TranslationFailed(String),

    #[error("File format error: {0}")]
    FileFormatError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, TranslatorError>;