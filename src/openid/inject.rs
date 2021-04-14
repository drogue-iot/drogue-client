use crate::{context::Context, error::ClientError, openid::OpenIdTokenProvider};
use async_trait::async_trait;
use reqwest::RequestBuilder;

#[async_trait]
pub trait TokenInjector {
    async fn inject_token(
        &self,
        builder: RequestBuilder,
        context: Context,
    ) -> Result<RequestBuilder, ClientError<reqwest::Error>>;
}

#[async_trait]
impl TokenInjector for Option<OpenIdTokenProvider> {
    async fn inject_token(
        &self,
        builder: RequestBuilder,
        context: Context,
    ) -> Result<RequestBuilder, ClientError<reqwest::Error>> {
        if let Some(token) = context.provided_token {
            Ok(builder.bearer_auth(token))
        } else if let Some(provider) = self {
            let token = provider.provide_access_token().await?;
            Ok(builder.bearer_auth(token))
        } else {
            Ok(builder)
        }
    }
}
