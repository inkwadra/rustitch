//! IRC adapter errors.

use rustitch_auth::AuthError;
use thiserror::Error;

/// Errors produced by the IRC adapter.
#[derive(Debug, Error)]
pub enum IrcError {
    /// IRC connection or reconnect failed.
    #[error("irc connection error: {0}")]
    Connection(String),

    /// IRC frame parsing or normalization failed.
    #[error("irc parse error: {0}")]
    Parse(String),

    /// Authentication failed.
    #[error(transparent)]
    Auth(#[from] AuthError),
}
