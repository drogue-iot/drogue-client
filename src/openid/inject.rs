use crate::openid::TokenProvider;
use crate::{context::Context, error::ClientError};
use async_trait::async_trait;

/// Allows injecting tokens.
#[async_trait(?Send)]
pub trait TokenInjector: Sized + Send + Sync {
    async fn inject_token(
        self,
        token_provider: &dyn TokenProvider<Error = reqwest::Error>,
        context: Context,
    ) -> Result<Self, ClientError<reqwest::Error>>;
}

/// Injects tokens into a request by setting the authorization header to a "bearer" token.
#[async_trait(?Send)]
impl TokenInjector for reqwest::RequestBuilder {
    async fn inject_token(
        self,
        token_provider: &dyn TokenProvider<Error = reqwest::Error>,
        context: Context,
    ) -> Result<Self, ClientError<reqwest::Error>> {
        if let Some(token) = context.provided_token {
            Ok(self.bearer_auth(token))
        } else if let Some(token) = token_provider.provide_access_token().await? {
            Ok(self.bearer_auth(token))
        } else {
            Ok(self)
        }
    }
}
