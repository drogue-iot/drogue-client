use crate::core::WithTracing;
use crate::openid::TokenProvider;
use crate::{error::ClientError, openid::TokenInjector};

use async_trait::async_trait;
use reqwest::{Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::Send;
use url::Url;

/// A drogue HTTP client, backed by reqwest.

#[async_trait]
pub trait Client<TP>
where
    TP: TokenProvider,
{
    /// Constructor
    fn new(client: reqwest::Client, url: Url, token_provider: TP) -> Self;

    /// Retrieve the http client
    fn client(&self) -> &reqwest::Client;

    /// Retrieve the token provider
    fn token_provider(&self) -> &TP;

    /// Execute a GET request to read a resouce content or to list resources
    ///
    /// The correct authentication and tracing headers will be added to the request.
    async fn read<T>(&self, url: Url) -> Result<Option<T>, ClientError<reqwest::Error>>
    where
        T: DeserializeOwned,
    {
        let req = self
            .client()
            .get(url)
            .propagate_current_context()
            .inject_token(self.token_provider())
            .await?;

        let response = req.send().await?;

        log::debug!("Eval get response: {:#?}", response);
        match response.status() {
            StatusCode::OK => Ok(Some(response.json().await?)),
            StatusCode::NOT_FOUND => Ok(None),
            _ => Self::default_response(response).await,
        }
    }

    /// Execute a PUT request to update an existing resource.
    ///
    /// A payload with the updated resource must be passed.
    /// The resource must exist, otherwise `false` is returned.
    ///
    /// The correct authentication and tracing headers will be added to the request.
    async fn update<A>(&self, url: Url, payload: A) -> Result<bool, ClientError<reqwest::Error>>
    where
        A: Serialize + Send + Sync,
    {
        let req = self
            .client()
            .put(url)
            .json(&payload)
            .propagate_current_context()
            .inject_token(self.token_provider())
            .await?;

        let response = req.send().await?;

        log::debug!("Eval update response: {:#?}", response);
        match response.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            _ => Self::default_response(response).await,
        }
    }

    /// Execute a DELETE request to delete an existing resource.
    ///
    /// The resource must exist, otherwise `false` is returned.
    ///
    /// The correct authentication and tracing headers will be added to the request.
    async fn delete(&self, url: Url) -> Result<bool, ClientError<reqwest::Error>> {
        let req = self
            .client()
            .delete(url)
            .inject_token(self.token_provider())
            .await?;

        let response = req.send().await?;

        log::debug!("Eval delete response: {:#?}", response);
        match response.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            _ => Self::default_response(response).await,
        }
    }

    /// Execute a POST request to create a resource.
    ///
    /// The correct authentication and tracing headers will be added to the request.
    async fn create<A, T>(
        &self,
        url: Url,
        payload: Option<A>,
    ) -> Result<Option<T>, ClientError<reqwest::Error>>
    where
        A: Serialize + Send + Sync,
        T: DeserializeOwned,
    {
        let req = if let Some(p) = payload {
            self.client().post(url).json(&p)
        } else {
            self.client().post(url)
        }
        .inject_token(self.token_provider())
        .await?;

        let response = req.send().await?;

        log::debug!("Eval create response: {:#?}", response);
        match response.status() {
            StatusCode::CREATED => Ok(Some(response.json().await?)),
            _ => Self::default_response(response).await,
        }
    }

    async fn default_response<T>(response: Response) -> Result<T, ClientError<reqwest::Error>> {
        match response.status() {
            code if code.is_client_error() => Err(ClientError::Service(response.json().await?)),
            code => Err(ClientError::Request(format!("Unexpected code {:?}", code))),
        }
    }
}
