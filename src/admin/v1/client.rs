use super::data::*;
use crate::error::ClientError;
use crate::openid::TokenProvider;
use crate::util::Client as TraitClient;
use std::fmt::Debug;
use tracing::instrument;
use url::Url;

/// A client for drogue cloud application administration API, backed by reqwest.
#[derive(Clone, Debug)]
pub struct Client<TP>
where
    TP: TokenProvider,
{
    client: reqwest::Client,
    api_url: Url,
    token_provider: TP,
}

enum AdministrationOperation {
    Transfer,
    Accept,
    Members,
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
        &self.token_provider
    }
}

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

    fn url(&self, application: &str, operation: AdministrationOperation) -> ClientResult<Url> {
        let mut url = self.api_url.clone();

        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| ClientError::Request("Failed to get paths".into()))?;

            path.extend(&["api", "admin", "v1alpha1", "apps"]);
            if !application.is_empty() {
                path.push(application);
            }
            match operation {
                AdministrationOperation::Transfer => path.push("transfer-ownership"),
                AdministrationOperation::Accept => path.push("accept-ownership"),
                AdministrationOperation::Members => path.push("members"),
            };
        }

        Ok(url)
    }

    /// Get the application members and their roles
    #[instrument]
    pub async fn get_members<A>(&self, application: A) -> ClientResult<Option<Members>>
    where
        A: AsRef<str> + Debug,
    {
        self.read(self.url(application.as_ref(), AdministrationOperation::Members)?)
            .await
    }

    /// Update the application members and their roles
    #[instrument]
    pub async fn update_members<A>(&self, application: A, members: Members) -> ClientResult<bool>
    where
        A: AsRef<str> + Debug,
    {
        self.update(
            self.url(application.as_ref(), AdministrationOperation::Members)?,
            Some(members),
        )
        .await
    }

    /// Transfer the application ownership to another user
    #[instrument]
    pub async fn initiate_app_transfer<A, U>(
        &self,
        application: A,
        username: U,
    ) -> ClientResult<bool>
    where
        A: AsRef<str> + Debug,
        U: AsRef<str> + Debug,
    {
        let payload = TransferOwnership {
            new_user: username.as_ref().to_string(),
        };

        self.update(
            self.url(application.as_ref(), AdministrationOperation::Transfer)?,
            Some(payload),
        )
        .await
    }

    /// Cancel the application ownership transfer
    #[instrument]
    pub async fn cancel_app_transfer<A>(&self, application: A) -> ClientResult<bool>
    where
        A: AsRef<str> + Debug,
    {
        self.delete(self.url(application.as_ref(), AdministrationOperation::Transfer)?)
            .await
    }

    /// Accept the application ownership transfer
    #[instrument]
    pub async fn accept_app_transfer<A>(&self, application: A) -> ClientResult<bool>
    where
        A: AsRef<str> + Debug,
    {
        self.update(
            self.url(application.as_ref(), AdministrationOperation::Accept)?,
            None::<()>,
        )
        .await
    }

    /// Read the application ownership transfer state
    #[instrument]
    pub async fn read_app_transfer<A>(
        &self,
        application: A,
    ) -> ClientResult<Option<TransferOwnership>>
    where
        A: AsRef<str> + Debug,
    {
        self.read(self.url(application.as_ref(), AdministrationOperation::Transfer)?)
            .await
    }
}
