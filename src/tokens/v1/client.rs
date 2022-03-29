use super::data::*;
use crate::error::ClientError;
use crate::openid::TokenProvider;
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
    pub async fn create_token(&self) -> ClientResult<Option<CreatedAccessToken>> {
        self.create(self.url(Some(""))?, None::<()>).await
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
