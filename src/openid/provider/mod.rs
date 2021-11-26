mod access_token;
mod openid;

pub use self::access_token::*;
pub use self::openid::*;

use crate::error::ClientError;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub enum Credentials {
    Bearer(String),
    Basic(String, Option<String>),
}

#[async_trait(?Send)]
pub trait TokenProvider {
    type Error: std::error::Error + Send + Sync;

    async fn provide_access_token(
        &self,
    ) -> Result<Option<Credentials>, crate::error::ClientError<Self::Error>>;
}

pub struct NoTokenProvider;

#[async_trait(?Send)]
impl TokenProvider for NoTokenProvider {
    type Error = reqwest::Error;

    async fn provide_access_token(&self) -> Result<Option<Credentials>, ClientError<Self::Error>> {
        Ok(None)
    }
}

#[async_trait(?Send)]
impl<T> TokenProvider for Option<T>
where
    T: TokenProvider + Sync,
{
    type Error = T::Error;

    async fn provide_access_token(&self) -> Result<Option<Credentials>, ClientError<Self::Error>> {
        match self {
            None => Ok(None),
            Some(provider) => provider.provide_access_token().await,
        }
    }
}
