use super::data::*;
use crate::{
    error::ClientError,
    openid::{OpenIdTokenProvider, TokenInjector},
    Context, Translator,
};
use futures::{stream, StreamExt, TryStreamExt};
use reqwest::{Response, StatusCode};
use serde::de::DeserializeOwned;
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

            path.extend(&["api", "v1", "apps"]);
            if !application.is_empty() {
                path.push(application);
            }

            if let Some(device) = device {
                path.push("devices");
                if !device.is_empty() {
                    path.push(device);
                }
            }
        }

        Ok(url)
    }

    /// Get an application by name.
    ///
    /// If the application do not exist, this function will return `None`, otherwise
    /// if will return the device information.
    ///
    /// If the user does not have access to the application, the server side may return "not found"
    /// as a response instead of "forbidden".
    pub async fn get_app<A>(
        &self,
        application: A,
        context: Context,
    ) -> ClientResult<Option<Application>>
    where
        A: AsRef<str>,
    {
        let req = self
            .client
            .get(self.url(application.as_ref(), None)?)
            .inject_token(&self.token_provider, context)
            .await?;

        Self::get_response(req.send().await?).await
    }

    /// Get a device by name.
    ///
    /// If the application or device do not exist, this function will return `None`, otherwise
    /// if will return the device information.
    ///
    /// If the user does not have access to the application, the server side may return "not found"
    /// as a response instead of "forbidden".
    pub async fn get_device<A, D>(
        &self,
        application: A,
        device: D,
        context: Context,
    ) -> ClientResult<Option<Device>>
    where
        A: AsRef<str>,
        D: AsRef<str>,
    {
        let req = self
            .client
            .get(self.url(application.as_ref(), Some(device.as_ref()))?)
            .inject_token(&self.token_provider, context)
            .await?;

        Self::get_response(req.send().await?).await
    }

    /// Get a list of devices.
    ///
    /// The function will only return devices that could be found.
    pub async fn get_devices<A, D>(
        &self,
        application: A,
        devices: &[D],
        context: Context,
    ) -> ClientResult<Vec<Device>>
    where
        A: AsRef<str>,
        D: AsRef<str>,
    {
        Ok(stream::iter(devices)
            .then(|device| self.get_device(application.as_ref(), device, context.clone()))
            // filter out missing devices
            .filter_map(|device| async { device.transpose() })
            // collect to a map
            .try_collect()
            .await?)
    }

    /// Get a device by name, resolving all first level gateways.
    pub async fn get_devices_and_gateways<A, D>(
        &self,
        application: A,
        device: D,
        context: Context,
    ) -> ClientResult<Option<(Device, Vec<Device>)>>
    where
        A: AsRef<str>,
        D: AsRef<str>,
    {
        let req = self
            .client
            .get(self.url(application.as_ref(), Some(device.as_ref()))?)
            .inject_token(&self.token_provider, context.clone())
            .await?;

        let device: Option<Device> = Self::get_response(req.send().await?).await?;

        if let Some(device) = device {
            let gateways = if let Some(gw_sel) = device
                .section::<DeviceSpecGatewaySelector>()
                .and_then(|s| s.ok())
            {
                // lookup devices
                self.get_devices(application, &gw_sel.match_names, context)
                    .await?
            } else {
                // unable to process gateways or no gateways configured
                vec![]
            };

            Ok(Some((device, gateways)))
        } else {
            Ok(None)
        }
    }

    async fn get_response<T: DeserializeOwned>(response: Response) -> ClientResult<Option<T>> {
        log::debug!("Eval get response: {:#?}", response);
        match response.status() {
            StatusCode::OK => Ok(Some(response.json().await?)),
            StatusCode::NOT_FOUND => Ok(None),
            _ => Self::default_response(response).await,
        }
    }

    /// Update (overwrite) an application.
    ///
    /// The application must exist, otherwise `false` is returned.
    pub async fn update_app(
        &self,
        application: Application,
        context: Context,
    ) -> ClientResult<bool> {
        let req = self
            .client
            .put(self.url(&application.metadata.name, None)?)
            .json(&application)
            .inject_token(&self.token_provider, context)
            .await?;

        Self::update_response(req.send().await?).await
    }

    /// Update (overwrite) a device.
    ///
    /// The application must exist, otherwise `false` is returned.
    pub async fn update_device(&self, device: Device, context: Context) -> ClientResult<bool> {
        let req = self
            .client
            .put(self.url(&device.metadata.application, Some(&device.metadata.name))?)
            .json(&device)
            .inject_token(&self.token_provider, context)
            .await?;

        Self::update_response(req.send().await?).await
    }

    async fn update_response(response: Response) -> ClientResult<bool> {
        log::debug!("Eval update response: {:#?}", response);
        match response.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            _ => Self::default_response(response).await,
        }
    }

    /// Create a new application.
    pub async fn create_app(&self, app: Application, context: Context) -> ClientResult<()> {
        let req = self
            .client
            .post(self.url("", None)?)
            .json(&app)
            .inject_token(&self.token_provider, context)
            .await?;

        Self::create_response(req.send().await?).await
    }

    /// Create a new device.
    pub async fn create_device(&self, device: Device, context: Context) -> ClientResult<()> {
        let req = self
            .client
            .post(self.url(&device.metadata.application, Some(""))?)
            .json(&device)
            .inject_token(&self.token_provider, context)
            .await?;

        Self::create_response(req.send().await?).await
    }

    async fn create_response(response: Response) -> ClientResult<()> {
        log::debug!("Eval create response: {:#?}", response);
        match response.status() {
            StatusCode::CREATED => Ok(()),
            _ => Self::default_response(response).await,
        }
    }

    async fn default_response<T>(response: Response) -> ClientResult<T> {
        match response.status() {
            code if code.is_client_error() => Err(ClientError::Service(response.json().await?)),
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
