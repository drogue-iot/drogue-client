use crate::{
    error::ClientError,
    openid::{Credentials, TokenProvider},
};
use async_trait::async_trait;
use reqwest_wasm_ext::ReqwestExt;
use tracing::instrument;

/// Allows injecting tokens.
#[async_trait]
pub trait TokenInjector: Sized + Send + Sync {
    async fn inject_token(self, token_provider: &dyn TokenProvider) -> Result<Self, ClientError>;
}

/// Injects tokens into a request by setting the authorization header to a "bearer" token.
#[async_trait]
impl TokenInjector for reqwest::RequestBuilder {
    #[instrument(skip(token_provider))]
    async fn inject_token(self, token_provider: &dyn TokenProvider) -> Result<Self, ClientError> {
        if let Some(credentials) = token_provider
            .provide_access_token()
            .await
            .map_err(|err| ClientError::Token(Box::new(err)))?
        {
            Ok(match credentials {
                Credentials::Bearer(token) => self.bearer_auth(token),
                Credentials::Basic(username, password) => self.basic_auth_ext(username, password),
            })
        } else {
            Ok(self)
        }
    }
}
