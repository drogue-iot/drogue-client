use crate::core::CoreClient;
use crate::error::ClientError;
use crate::openid::TokenProvider;
use serde::Serialize;
use std::{fmt::Debug, sync::Arc};
use tracing::instrument;
use url::Url;

/// A client for drogue cloud command and control API, backed by reqwest.
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

    fn url(&self, application: &str, device: &str) -> ClientResult<Url> {
        let mut url = self.api_url.clone();

        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| ClientError::Request("Failed to get paths".into()))?;

            path.extend(&[
                "api",
                "command",
                "v1alpha1",
                "apps",
                application,
                "devices",
                device,
            ]);
        }

        Ok(url)
    }

    /// Send one way commands to devices.
    ///
    /// The result will be true if the command was accepted.
    /// False
    #[instrument(skip(payload))]
    pub async fn publish_command<A, D, C, P>(
        &self,
        application: A,
        device: D,
        command: C,
        payload: Option<P>,
    ) -> ClientResult<Option<()>>
    where
        A: AsRef<str> + Debug,
        D: AsRef<str> + Debug,
        C: AsRef<str> + Debug,
        P: Serialize + Send + Sync,
    {
        let url = self.url(application.as_ref(), device.as_ref())?;
        let query = vec![("command".to_string(), command.as_ref().to_string())];

        self.create_with_query_parameters(url, payload, Some(query))
            .await
    }
}
