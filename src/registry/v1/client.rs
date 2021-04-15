use super::data::*;
use crate::{
    error::ClientError,
    openid::{OpenIdTokenProvider, TokenInjector},
    Context,
};
use reqwest::StatusCode;
use url::Url;

/// A device registry client backed by reqwest.
#[derive(Clone, Debug)]
pub struct Client {
    client: reqwest::Client,
    device_registry_url: Url,
    token_provider: Option<OpenIdTokenProvider>,
}

type ClientResult<T> = Result<T, ClientError<reqwest::Error>>;

impl Client {
    /// Create a new client instance.
    pub fn new(
        client: reqwest::Client,
        device_registry_url: Url,
        token_provider: Option<OpenIdTokenProvider>,
    ) -> Self {
        Self {
            client,
            device_registry_url,
            token_provider,
        }
    }

    fn url(&self, application: &str, device: &str) -> ClientResult<Url> {
        Ok(self
            .device_registry_url
            .join(&format!("/api/v1/apps/{}/devices/{}", application, device))?)
    }

    /// Get a device by name, returning the raw JSON.
    pub async fn get_device(
        &self,
        application: &str,
        device: &str,
        context: Context,
    ) -> ClientResult<Option<Device>> {
        let req = self
            .client
            .get(self.url(application, device)?)
            .inject_token(&self.token_provider, context)
            .await?;

        let response = req.send().await?;

        log::debug!("Response: {:#?}", response);

        match response.status() {
            StatusCode::OK => Ok(Some(response.json().await?)),
            StatusCode::NOT_FOUND => Ok(None),
            code => Err(ClientError::Request(format!("Unexpected code {:?}", code))),
        }
    }
}
