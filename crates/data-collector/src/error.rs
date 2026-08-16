//! Error types for the data collector

use thiserror::Error;

/// Errors that can occur during data collection
#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum CollectorError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("YAML serialization error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("URL error: {0}")]
    #[allow(dead_code)]
    UrlError(String),
}

/// Result type for the data collector
pub type Result<T> = std::result::Result<T, CollectorError>;
