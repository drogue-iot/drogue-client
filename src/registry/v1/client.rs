use super::data::*;
use crate::{
    error::ClientError,
    openid::{OpenIdTokenProvider, TokenInjector},
    Context,
};
use reqwest::StatusCode;
use url::Url;

/// A device registry client, backed by reqwest.
#[derive(Clone, Debug)]
pub struct Client {
    client: reqwest::Client,
    registry_url: Url,
    token_provider: Option<OpenIdTokenProvider>,
}

type ClientResult<T> = Result<T, ClientError<reqwest::Error>>;

impl Client {
    /// Create a new client instance.
    pub fn new(
        client: reqwest::Client,
        registry_url: Url,
        token_provider: Option<OpenIdTokenProvider>,
    ) -> Self {
        Self {
            client,
            registry_url,
            token_provider,
        }
    }

    fn url(&self, application: &str, device: Option<&str>) -> ClientResult<Url> {
        let mut url = self.registry_url.clone();

        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| ClientError::Request("Failed to get paths".into()))?;

            path.extend(&["api", "v1", "apps", application]);

            if let Some(device) = device {
                path.extend(&["devices", device]);
            }
        }

        Ok(url)
    }

    /// Get an application by name, returning the raw JSON.
    ///
    /// If the application do not exist, this function will return `None`, otherwise
    /// if will return the device information.
    ///
    /// If the user does not have access to the application, the server side may return "not found"
    /// as a response instead of "forbidden".
    pub async fn get_app(
        &self,
        application: &str,
        context: Context,
    ) -> ClientResult<Option<Device>> {
        let req = self
            .client
            .get(self.url(application.as_ref(), None)?)
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

    /// Get a device by name, returning the raw JSON.
    ///
    /// If the application or device do not exist, this function will return `None`, otherwise
    /// if will return the device information.
    ///
    /// If the user does not have access to the application, the server side may return "not found"
    /// as a response instead of "forbidden".
    pub async fn get_device<S1, S2>(
        &self,
        application: S1,
        device: S2,
        context: Context,
    ) -> ClientResult<Option<Device>>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        let req = self
            .client
            .get(self.url(application.as_ref(), Some(device.as_ref()))?)
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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_url_app() -> anyhow::Result<()> {
        let client = Client::new(Default::default(), Url::parse("http://localhost")?, None);

        let url = client.url("foo", Some("bar/baz")).unwrap();
        assert_eq!(
            url.to_string(),
            "http://localhost/api/v1/apps/foo/devices/bar%2Fbaz"
        );

        Ok(())
    }
}
