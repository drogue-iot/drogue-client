use crate::{
    error::ClientError,
    openid::{Credentials, TokenProvider},
};
use async_trait::async_trait;

/// A token provider, using an API key as static token.
pub struct ApiKeyProvider {
    pub user: String,
    pub key: String,
}

#[async_trait]
impl TokenProvider for ApiKeyProvider {
    type Error = reqwest::Error;

    async fn provide_access_token(&self) -> Result<Option<Credentials>, ClientError<Self::Error>> {
        Ok(Some(Credentials::Basic(
            self.user.clone(),
            Some(self.key.clone()),
        )))
    }
}
