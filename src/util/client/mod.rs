use crate::core::PropagateCurrentContext;
use crate::openid::TokenProvider;
use crate::{error::ClientError, error::ErrorInformation, openid::TokenInjector};

use async_trait::async_trait;
use reqwest::{Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::Send;
use url::Url;

/// A drogue HTTP client, backed by reqwest.

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Client {
    /// Retrieve the http client
    fn client(&self) -> &reqwest::Client;

    /// Retrieve the token provider
    fn token_provider(&self) -> &dyn TokenProvider;

    /// Execute a GET request to read a resource content or to list resources
    ///
    /// The correct authentication and tracing headers will be added to the request.
    #[doc(hidden)]
    async fn read<T>(&self, url: Url) -> Result<Option<T>, ClientError>
    where
        Self: Send,
        T: DeserializeOwned,
    {
        self.read_with_query_parameters(url, None).await
    }

    /// Execute a GET request to read a resource content or to list resources
    /// Optionally add query parameters.
    ///
    /// The correct authentication and tracing headers will be added to the request.
    #[doc(hidden)]
    async fn read_with_query_parameters<T>(
        &self,
        url: Url,
        query: Option<Vec<(String, String)>>,
    ) -> Result<Option<T>, ClientError>
    where
        Self: Send,
        T: DeserializeOwned,
    {
        let query = query.unwrap_or_default();

        let req = self
            .client()
            .get(url)
            .query(&query)
            .propagate_current_context()
            .inject_token(self.token_provider())
            .await?;

        Self::read_response(req.send().await?).await
    }

    async fn read_response<T: DeserializeOwned>(
        response: Response,
    ) -> Result<Option<T>, ClientError> {
        log::debug!("Eval get response: {:#?}", response);
        match response.status() {
            StatusCode::OK => Ok(Some(response.json().await?)),
            StatusCode::NOT_FOUND => Ok(None),
            _ => Self::default_response(response).await,
        }
    }

    /// Execute a PUT request to update an existing resource.
    ///
    /// A payload with the updated resource can be passed.
    /// The resource must exist, otherwise `false` is returned.
    ///
    /// The correct authentication and tracing headers will be added to the request.
    #[doc(hidden)]
    async fn update<A>(&self, url: Url, payload: Option<A>) -> Result<bool, ClientError>
    where
        Self: Send,
        A: Serialize + Send + Sync,
    {
        let req = if let Some(p) = payload {
            self.client().put(url).json(&p)
        } else {
            self.client().put(url)
        }
        .propagate_current_context()
        .inject_token(self.token_provider())
        .await?;

        Self::update_response(req.send().await?).await
    }

    async fn update_response(response: Response) -> Result<bool, ClientError> {
        log::debug!("Eval update response: {:#?}", response);
        match response.status() {
            StatusCode::OK | StatusCode::NO_CONTENT | StatusCode::ACCEPTED => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            _ => Self::default_response(response).await,
        }
    }

    /// Execute a DELETE request to delete an existing resource.
    ///
    /// The resource must exist, otherwise `false` is returned.
    ///
    /// The correct authentication and tracing headers will be added to the request.
    #[doc(hidden)]
    async fn delete(&self, url: Url) -> Result<bool, ClientError>
    where
        Self: Send,
    {
        let req = self
            .client()
            .delete(url)
            .inject_token(self.token_provider())
            .await?;

        Self::delete_response(req.send().await?).await
    }

    async fn delete_response(response: Response) -> Result<bool, ClientError> {
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
    #[doc(hidden)]
    async fn create<P, T>(&self, url: Url, payload: Option<P>) -> Result<Option<T>, ClientError>
    where
        Self: std::marker::Send,
        P: Serialize + Send + Sync,
        T: DeserializeOwned,
    {
        self.create_with_query_parameters(url, payload, None).await
    }

    /// Execute a POST request to create a resource.
    /// Optionally add query parameters
    ///
    /// The correct authentication and tracing headers will be added to the request.
    #[doc(hidden)]
    async fn create_with_query_parameters<P, T>(
        &self,
        url: Url,
        payload: Option<P>,
        query: Option<Vec<(String, String)>>,
    ) -> Result<Option<T>, ClientError>
    where
        Self: std::marker::Send,
        P: Serialize + Send + Sync,
        T: DeserializeOwned,
    {
        let query = query.unwrap_or_default();

        let req = if let Some(p) = payload {
            self.client().post(url).json(&p)
        } else {
            self.client().post(url)
        }
        .query(&query)
        .propagate_current_context()
        .inject_token(self.token_provider())
        .await?;

        Self::create_response(req.send().await?).await
    }

    async fn create_response<T: DeserializeOwned>(
        response: Response,
    ) -> Result<Option<T>, ClientError> {
        log::debug!("Eval create response: {:#?}", response);
        match response.status() {
            StatusCode::CREATED | StatusCode::ACCEPTED => Ok(None),
            // the token API responds 200 on token creations, sending back the content.
            StatusCode::OK => Ok(Some(response.json().await?)),
            _ => Self::default_response(response).await,
        }
    }

    async fn default_response<T>(response: Response) -> Result<T, ClientError> {
        match response.status() {
            code if code.is_client_error() => {
                let error = match response.json().await {
                    Ok(json) => ErrorInformation {
                        error: json,
                        message: format!("HTTP {}", code),
                        status: code,
                    },
                    Err(_) => ErrorInformation {
                        error: String::default(),
                        message: format!("HTTP error {}", code),
                        status: code,
                    },
                };
                Err(ClientError::Service(error))
            }
            code => Err(ClientError::Service(ErrorInformation {
                error: String::default(),
                message: format!("Unexpected HTTP code {:?}", code),
                status: code,
            })),
        }
    }
}
