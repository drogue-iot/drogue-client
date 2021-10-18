use crate::{error::ClientError, openid::TokenProvider};
use async_trait::async_trait;

/// A token provider, using an API key as static token.
pub struct ApiKeyProvider(pub String);

#[async_trait]
impl TokenProvider for ApiKeyProvider {
    type Error = reqwest::Error;

    async fn provide_access_token(&self) -> Result<Option<String>, ClientError<Self::Error>> {
        Ok(Some(self.0.clone()))
    }
}
