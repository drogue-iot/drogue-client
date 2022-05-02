use crate::{
    error::ClientError,
    openid::{Credentials, TokenProvider},
};
use async_trait::async_trait;
use std::fmt::{Debug, Formatter};

/// A token provider, using an Access Token as static token.
#[derive(Clone)]
pub struct AccessTokenProvider {
    pub user: String,
    pub token: String,
}

impl Debug for AccessTokenProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccessTokenProvider")
            .field("user", &self.user)
            .field("token", &"***")
            .finish()
    }
}

#[async_trait]
impl TokenProvider for AccessTokenProvider {
    async fn provide_access_token(&self) -> Result<Option<Credentials>, ClientError> {
        Ok(Some(Credentials::Basic(
            self.user.clone(),
            Some(self.token.clone()),
        )))
    }
}
