use super::data::*;
use crate::openid::TokenProvider;
use crate::{error::ClientError, openid::TokenInjector, Context};
use reqwest::{Response, StatusCode};
use serde::de::DeserializeOwned;
use url::Url;

/// A device registry client, backed by reqwest.
#[derive(Clone, Debug)]
pub struct Client<TP>
where
    TP: TokenProvider,
{
    client: reqwest::Client,
    api_url: Url,
    token_provider: TP,
}

type ClientResult<T> = Result<T, ClientError<reqwest::Error>>;

impl<TP> Client<TP>
where
    TP: TokenProvider,
{
    /// Create a new client instance.
    pub fn new(client: reqwest::Client, api_url: Url, token_provider: TP) -> Self {
        Self {
            client,
            api_url,
            token_provider,
        }
    }

    fn url(&self, prefix: &str) -> ClientResult<Url> {
        let mut url = self.api_url.clone();

        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| ClientError::Request("Failed to get paths".into()))?;

            path.extend(&["api", "tokens", "v1alpha1"]);
            if !prefix.is_empty() {
                path.push(prefix);
            }
        }

        Ok(url)
    }

    /// Get a list of active access tokens for this user.
    ///
    /// The full token won't be disclosed, as it is secret and unknown by the server.
    /// The result contains the prefix and creation date for each active token.
    pub async fn get_tokens(&self, context: Context) -> ClientResult<Vec<AccessToken>> {
        let req = self
            .client
            .get(self.url("")?)
            .inject_token(&self.token_provider, context)
            .await?;

        Self::get_response(req.send().await?).await
    }

    async fn get_response<T: DeserializeOwned>(response: Response) -> ClientResult<Vec<T>> {
        log::debug!("Eval get response: {:#?}", response);
        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            StatusCode::NOT_FOUND => Ok(Vec::new()),
            _ => Self::default_response(response).await,
        }
    }

    /// Create a new access token for this user.
    ///
    /// The result will contain the full token. This value is only available once.
    pub async fn create_token(&self, context: Context) -> ClientResult<CreatedAccessToken> {
        let req = self
            .client
            .post(self.url("")?)
            .inject_token(&self.token_provider, context)
            .await?;

        Self::create_response(req.send().await?).await
    }

    async fn create_response<T: DeserializeOwned>(response: Response) -> ClientResult<T> {
        log::debug!("Eval create response: {:#?}", response);
        match response.status() {
            StatusCode::CREATED => Ok(response.json().await?),
            _ => Self::default_response(response).await,
        }
    }

    // TODO : refactor - it already exists in registry::client
    pub async fn delete_token<A>(&self, prefix: A, context: Context) -> ClientResult<bool>
    where
        A: AsRef<str>,
    {
        let req = self
            .client
            .delete(self.url(prefix.as_ref())?)
            .inject_token(&self.token_provider, context)
            .await?;

        Self::delete_response(req.send().await?).await
    }

    // TODO : refactor - it already exists in registry::client
    async fn delete_response(response: Response) -> ClientResult<bool> {
        log::debug!("Eval delete response: {:#?}", response);
        match response.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            _ => Self::default_response(response).await,
        }
    }

    // TODO : refactor - it already exists in registry::client
    async fn default_response<T>(response: Response) -> ClientResult<T> {
        match response.status() {
            code if code.is_client_error() => Err(ClientError::Service(response.json().await?)),
            code => Err(ClientError::Request(format!("Unexpected code {:?}", code))),
        }
    }
}
