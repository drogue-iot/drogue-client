use crate::openid::{Credentials, TokenProvider};
use crate::{context::Context, error::ClientError};
use async_trait::async_trait;

/// Allows injecting tokens.
#[async_trait]
pub trait TokenInjector: Sized + Send + Sync {
    async fn inject_token<TP>(
        self,
        token_provider: &TP,
        context: Context,
    ) -> Result<Self, ClientError<reqwest::Error>>
    where
        TP: TokenProvider;
}

/// Injects tokens into a request by setting the authorization header to a "bearer" token.
#[async_trait]
impl TokenInjector for reqwest::RequestBuilder {
    async fn inject_token<TP>(
        self,
        token_provider: &TP,
        context: Context,
    ) -> Result<Self, ClientError<reqwest::Error>>
    where
        TP: TokenProvider,
    {
        if let Some(token) = context.provided_token {
            Ok(self.bearer_auth(token))
        } else if let Some(credentials) = token_provider
            .provide_access_token()
            .await
            .map_err(|err| ClientError::Token(Box::new(err)))?
        {
            Ok(match credentials {
                Credentials::Bearer(token) => self.bearer_auth(token),
                Credentials::Basic(username, password) => self.basic_auth(username, password),
            })
        } else {
            Ok(self)
        }
    }
}
