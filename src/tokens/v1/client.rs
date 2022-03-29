use super::data::*;
use crate::error::ClientError;
use crate::openid::{TokenInjector, TokenProvider};
use crate::util::Client;
use std::fmt::Debug;
use tracing::instrument;
use url::Url;

/// A device registry client, backed by reqwest.
#[derive(Clone, Debug)]
pub struct TokenClient<TP>
where
    TP: TokenProvider,
{
    client: reqwest::Client,
    api_url: Url,
    token_provider: TP,
}

type ClientResult<T> = Result<T, ClientError<reqwest::Error>>;

impl<TP> Client<TP> for TokenClient<TP>
where
    TP: TokenProvider,
{
    /// Create a new client instance.
    fn new(client: reqwest::Client, api_url: Url, token_provider: TP) -> Self {
        Self {
            client,
            api_url,
            token_provider,
        }
    }

    fn client(&self) -> &reqwest::Client {
        &self.client
    }

    fn token_provider(&self) -> &TP {
        &self.token_provider
    }
}

impl<TP> TokenClient<TP>
where
    TP: TokenProvider,
{
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
        // here we need to override the trait method as we need to add a query parameter with the description.
        let url = self.url(Some(""))?;

        let mut query: Vec<(&str, &str)> = Vec::new();
        if let Some(description) = description.as_ref() {
            query.push(("description", description.as_ref()))
        }

        let req = self
            .client()
            .post(url)
            .query(&query)
            .inject_token(self.token_provider())
            .await?;

        Self::create_response(req.send().await?).await
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
