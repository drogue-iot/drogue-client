mod access_token;
#[cfg(feature = "openid")]
mod openid;

pub use self::access_token::*;
#[cfg(feature = "openid")]
pub use self::openid::*;

use crate::error::ClientError;
use async_trait::async_trait;
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub enum Credentials {
    Bearer(String),
    Basic(String, Option<String>),
}

#[async_trait]
pub trait TokenProvider: Send + Sync + Debug {
    async fn provide_access_token(&self) -> Result<Option<Credentials>, ClientError>;
}

#[derive(Debug, Clone, Copy)]
pub struct NoTokenProvider;

#[async_trait]
impl TokenProvider for NoTokenProvider {
    async fn provide_access_token(&self) -> Result<Option<Credentials>, ClientError> {
        Ok(None)
    }
}

#[async_trait]
impl<T> TokenProvider for Option<T>
where
    T: TokenProvider + Sync,
{
    async fn provide_access_token(&self) -> Result<Option<Credentials>, ClientError> {
        match self {
            None => Ok(None),
            Some(provider) => provider.provide_access_token().await,
        }
    }
}

#[async_trait]
impl TokenProvider for String {
    async fn provide_access_token(&self) -> Result<Option<Credentials>, ClientError> {
        Ok(Some(Credentials::Bearer(self.clone())))
    }
}
