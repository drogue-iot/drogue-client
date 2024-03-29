use super::data::*;
use crate::core::CoreClient;
use crate::error::ClientError;
use crate::openid::TokenProvider;
use std::{fmt::Debug, sync::Arc};
use tracing::instrument;
use url::Url;

/// A client for the token management API, backed by reqwest.
#[derive(Clone, Debug)]
pub struct Client {
    client: reqwest::Client,
    api_url: Url,
    token_provider: Arc<dyn TokenProvider>,
}

type ClientResult<T> = Result<T, ClientError>;

impl CoreClient for Client {
    fn client(&self) -> &reqwest::Client {
        &self.client
    }

    fn token_provider(&self) -> &dyn TokenProvider {
        self.token_provider.as_ref()
    }
}

impl Client {
    /// Create a new client instance.
    pub fn new(
        client: reqwest::Client,
        api_url: Url,
        token_provider: impl TokenProvider + 'static,
    ) -> Self {
        Self {
            client,
            api_url,
            token_provider: Arc::new(token_provider),
        }
    }

    fn url(&self, prefix: Option<&str>) -> ClientResult<Url> {
        let mut url = self.api_url.clone();

        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| ClientError::Request("Failed to get paths".into()))?;

            path.extend(&["api", "tokens", "v1alpha1"]);
            if let Some(prefix) = prefix {
                if !prefix.is_empty() {
                    path.push(prefix);
                }
            }
        }

        Ok(url)
    }

    /// Get a list of active access tokens for this user.
    ///
    /// The full token won't be disclosed, as it is secret and unknown by the server.
    /// The result contains the prefix and creation date for each active token.
    #[instrument]
    pub async fn get_tokens(&self) -> ClientResult<Option<Vec<AccessToken>>> {
        self.read(self.url(Some(""))?).await
    }

    /// Create a new access token for this user.
    ///
    /// The result will contain the full token. This value is only available once.
    #[instrument]
    pub async fn create_token<D>(
        &self,
        description: Option<D>,
    ) -> ClientResult<Option<CreatedAccessToken>>
    where
        D: AsRef<str> + Debug,
    {
        let url = self.url(Some(""))?;

        let param =
            description.map(move |d| vec![("description".to_string(), d.as_ref().to_string())]);

        self.create_with_query_parameters(url, None::<()>, param)
            .await
    }

    /// Delete an existing token for this user.
    #[instrument]
    pub async fn delete_token<P>(&self, prefix: P) -> ClientResult<bool>
    where
        P: AsRef<str> + Debug,
    {
        self.delete(self.url(Some(prefix.as_ref()))?).await
    }
}
