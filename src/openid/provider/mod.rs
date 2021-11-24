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

#[async_trait]
pub trait TokenProvider {
    type Error: std::error::Error + Send + Sync;

    async fn provide_access_token(
        &self,
    ) -> Result<Option<Credentials>, crate::error::ClientError<Self::Error>>;
}

pub struct NoTokenProvider;

#[async_trait]
impl TokenProvider for NoTokenProvider {
    type Error = reqwest::Error;

    async fn provide_access_token(&self) -> Result<Option<Credentials>, ClientError<Self::Error>> {
        Ok(None)
    }
}
