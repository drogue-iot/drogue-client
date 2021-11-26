use crate::{
    error::ClientError,
    openid::{Credentials, TokenProvider},
};
use async_trait::async_trait;

/// A token provider, using an Access Token as static token.
pub struct AccessTokenProvider {
    pub user: String,
    pub token: String,
}

#[async_trait(?Send)]
impl TokenProvider for AccessTokenProvider {
    type Error = reqwest::Error;

    async fn provide_access_token(&self) -> Result<Option<Credentials>, ClientError<Self::Error>> {
        Ok(Some(Credentials::Basic(
            self.user.clone(),
            Some(self.token.clone()),
        )))
    }
}
