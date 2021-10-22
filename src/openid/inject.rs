use crate::openid::{Credentials, TokenProvider};
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
        } else if let Some(credentials) = token_provider.provide_access_token().await? {
            Ok(match credentials {
                Credentials::Bearer(token) => self.bearer_auth(token),
                Credentials::Basic(username, password) => self.basic_auth(username, password),
            })
        } else {
            Ok(self)
        }
    }
}
