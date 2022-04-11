use super::data::*;
use crate::error::ClientError;
use crate::openid::TokenProvider;
use crate::util::Client as TraitClient;
use std::fmt::Debug;
use tracing::instrument;
use url::Url;

/// A client to discover available drogue-cloud endpoints and their URL.
#[derive(Clone, Debug)]
pub struct Client<TP>
where
    TP: TokenProvider,
{
    client: reqwest::Client,
    api_url: Url,
    token_provider: Option<TP>,
}

type ClientResult<T> = Result<T, ClientError<reqwest::Error>>;

impl<TP> TraitClient<TP> for Client<TP>
where
    TP: TokenProvider,
{
    fn client(&self) -> &reqwest::Client {
        &self.client
    }

    fn token_provider(&self) -> &TP {
        self.token_provider.as_ref().unwrap()
    }
}

impl<TP> Client<TP>
where
    TP: TokenProvider,
{
    /// Create a new unauthenticated client instance.
    pub fn new_anonymous(client: reqwest::Client, api_url: Url) -> Self {
        Self {
            client,
            api_url,
            token_provider: None,
        }
    }

    /// Create a new authenticated client instance.
    pub fn new_authenticated(client: reqwest::Client, api_url: Url, token_provider: TP) -> Self {
        Self {
            client,
            api_url,
            token_provider: Some(token_provider),
        }
    }

    fn url(&self, authenticated: bool) -> ClientResult<Url> {
        let mut url = self.api_url.clone();

        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| ClientError::Request("Failed to get paths".into()))?;

            if authenticated {
                if let Some(_) = self.token_provider {
                    path.extend(&["api", "console", "v1alpha1", "info"]);
                } else {
                    return Err(ClientError::Request(
                        "No token provider, the client is not authenticated.".into(),
                    ));
                }
            } else {
                path.extend(&[".well-known", "drogue-endpoints"]);
            }
        }

        Ok(url)
    }

    /// Fetch drogue's well known endpoint to retrieve a list of accessible endpoints.
    /// This endpoint does not require authentication, therefore the returned list of endpoint is not complete.
    #[instrument]
    pub async fn get_public_endpoints(&self) -> ClientResult<Option<Endpoints>> {
        let req = self.client().get(self.url(false)?);

        Self::read_response(req.send().await?).await
    }

    /// Fetch drogue full list of accessible endpoints.
    #[instrument]
    pub async fn get_authenticated_endpoints(&self) -> ClientResult<Option<Endpoints>> {
        self.read(self.url(true)?).await
    }

    /// Fetch drogue-cloud running version.
    #[instrument]
    pub async fn get_drogue_cloud_version(&self) -> ClientResult<Option<DrogueVersion>> {
        let url = self.api_url.join(".well-known/drogue-version")?;
        let req = self.client().get(url);

        Self::read_response(req.send().await?).await
    }

    /// Fetch drogue-cloud Single Sign On provider URL.
    #[instrument]
    pub async fn get_sso_url(&self) -> ClientResult<Option<Url>> {
        self.get_authenticated_endpoints().await.map(|r| {
            r.map(|endpoints| {
                endpoints
                    .issuer_url
                    .map(|url| Url::parse(url.as_str()).ok())
                    .flatten()
            })
            .flatten()
        })
    }
}

#[cfg(test)]
mod test {
    use crate::discovery::v1::Client;
    use crate::openid::NoTokenProvider;
    use url::Url;

    #[tokio::test]
    async fn test_get_drogue_version() {
        let client: Client<NoTokenProvider> = Client::new_anonymous(
            reqwest::Client::new(),
            Url::parse("https://api.sandbox.drogue.cloud").unwrap(),
        );

        let version = client.get_drogue_cloud_version().await;
        assert!(version.is_ok());
        let version = version.unwrap();

        assert!(version.is_some());
        let version = version.unwrap();
        assert!(!version.version.is_empty());
    }

    #[tokio::test]
    async fn test_get_drogue_public_endpoints() {
        let client: Client<NoTokenProvider> = Client::new_anonymous(
            reqwest::Client::new(),
            Url::parse("https://api.sandbox.drogue.cloud").unwrap(),
        );

        let endpoints = client.get_public_endpoints().await;
        assert!(endpoints.is_ok());
        let endpoints = endpoints.unwrap();

        assert!(endpoints.is_some());
        let endpoints = endpoints.unwrap();

        assert!(endpoints.issuer_url.is_some());
        assert!(endpoints.api.is_some());
        assert!(endpoints.registry.is_some());
        assert!(endpoints.sso.is_some());

        assert!(endpoints.http.is_none());
        assert!(endpoints.mqtt.is_none());
        assert!(endpoints.kafka_bootstrap_servers.is_none());
        assert!(endpoints.mqtt_integration.is_none());
    }
}
