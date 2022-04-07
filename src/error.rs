use serde::{Deserialize, Serialize};
use std::fmt;
use url::ParseError;

/// Additional error information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorInformation {
    /// A machine processable error type.
    pub error: String,
    /// A human readable error message.
    #[serde(default)]
    pub message: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    /// An error from the underlying API client (e.g. reqwest).
    #[error("client error: {0}")]
    Client(#[from] Box<E>),
    /// A local error, performing the request.
    #[error("request error: {0}")]
    Request(String),
    /// A remote error, performing the request.
    #[error("service error: {0}")]
    Service(ErrorInformation),
    /// A token provider error.
    #[error("token error: {0}")]
    Token(#[source] Box<dyn std::error::Error + Send + Sync>),
    /// Url error.
    #[error("Url parse error")]
    Url(#[from] ParseError),
    /// Syntax error.
    #[error("Syntax error: {0}")]
    Syntax(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl<E> ClientError<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    pub fn syntax<S>(err: S) -> ClientError<E>
    where
        S: std::error::Error + Send + Sync + 'static,
    {
        Self::Syntax(Box::new(err))
    }
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for ClientError<reqwest::Error> {
    fn from(err: reqwest::Error) -> Self {
        ClientError::Client(Box::new(err))
    }
}

impl<E> From<serde_json::Error> for ClientError<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(err: serde_json::Error) -> Self {
        ClientError::Syntax(Box::new(err))
    }
}

impl fmt::Display for ErrorInformation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.error.is_empty() {
            write!(f, "{}", self.message)
        } else {
            write!(f, "{}: {}", self.error, self.message)
        }
    }
}
