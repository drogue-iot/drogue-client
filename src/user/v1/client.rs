use super::{authn, authz};
use crate::{core::CoreClient, error::ClientError, openid::TokenProvider};
use std::sync::Arc;
use tracing::instrument;
use url::Url;

#[cfg(feature = "telemetry")]
use crate::metrics::PassFailErrorExt;

#[cfg(feature = "telemetry")]
lazy_static::lazy_static! {
    pub static ref AUTHENTICATION: prometheus::IntGaugeVec = prometheus::register_int_gauge_vec!(
        "drogue_client_user_authentication_access_token",
        "User access token authentication operations",
        &["outcome"]
    )
    .unwrap();
    pub static ref AUTHORIZATION: prometheus::IntGaugeVec = prometheus::register_int_gauge_vec!(
        "drogue_client_user_authorization",
        "User access authorization",
        &["outcome"]
    )
    .unwrap();
}

/// A client for authorizing user requests.
#[derive(Clone, Debug)]
pub struct Client {
    client: reqwest::Client,
    authn_url: Url,
    authz_url: Url,
    token_provider: Arc<dyn TokenProvider>,
}

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
        authn_url: Url,
        authz_url: Url,
        token_provider: impl TokenProvider + 'static,
    ) -> Self {
        Self {
            client,
            authn_url,
            authz_url,
            token_provider: Arc::new(token_provider),
        }
    }

    #[allow(clippy::let_and_return)]
    #[instrument]
    pub async fn authenticate_access_token(
        &self,
        request: authn::AuthenticationRequest,
    ) -> Result<authn::AuthenticationResponse, ClientError> {
        let resp = self
            .create(self.authn_url.clone(), Some(&request))
            .await?
            .ok_or_else(|| ClientError::UnexpectedResponse("Missing response payload".to_string()));

        #[cfg(feature = "telemetry")]
        let resp = resp.record_outcome(&AUTHENTICATION);

        resp
    }

    #[allow(clippy::let_and_return)]
    #[instrument]
    pub async fn authorize(
        &self,
        request: authz::AuthorizationRequest,
    ) -> Result<authz::AuthorizationResponse, ClientError> {
        let resp = self
            .create(self.authz_url.clone(), Some(&request))
            .await?
            .ok_or_else(|| ClientError::UnexpectedResponse("Missing response payload".to_string()));

        #[cfg(feature = "telemetry")]
        let resp = resp.record_outcome(&AUTHORIZATION);

        resp
    }
}
