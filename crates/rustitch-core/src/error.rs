//! Common error taxonomy shared across the `rustitch` workspace.

use thiserror::Error;

/// Top-level error type for shared infrastructure and validation failures.
#[derive(Debug, Error)]
pub enum CoreError {
    /// A required configuration value is missing or malformed.
    #[error("configuration error: {0}")]
    Configuration(String),

    /// An identifier failed validation.
    #[error("invalid identifier: {0}")]
    InvalidIdentifier(String),

    /// A secret value failed validation.
    #[error("invalid secret: {0}")]
    InvalidSecret(String),
}
