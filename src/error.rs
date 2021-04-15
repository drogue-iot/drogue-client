use serde::{Deserialize, Serialize};
use std::fmt;
use url::ParseError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorInformation {
    pub error: String,
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
    Syntax(#[from] serde_json::error::Error),
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for ClientError<reqwest::Error> {
    fn from(err: reqwest::Error) -> Self {
        ClientError::Client(Box::new(err))
    }
}

impl fmt::Display for ErrorInformation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error, self.message)
    }
}
