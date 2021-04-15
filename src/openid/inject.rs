use crate::{context::Context, error::ClientError, openid::OpenIdTokenProvider};
use async_trait::async_trait;

/// Allows injecting tokens.
#[async_trait]
pub trait TokenInjector: Sized {
    async fn inject_token(
        self,
        token_provider: &Option<OpenIdTokenProvider>,
        context: Context,
    ) -> Result<Self, ClientError<reqwest::Error>>;
}

/// Injects tokens into a request by setting the authorization header to a "bearer" token.
#[async_trait]
impl TokenInjector for reqwest::RequestBuilder {
    async fn inject_token(
        self,
        token_provider: &Option<OpenIdTokenProvider>,
        context: Context,
    ) -> Result<Self, ClientError<reqwest::Error>> {
        if let Some(token) = context.provided_token {
            Ok(self.bearer_auth(token))
        } else if let Some(provider) = token_provider {
            let token = provider.provide_access_token().await?;
            Ok(self.bearer_auth(token))
        } else {
            Ok(self)
        }
    }
}
