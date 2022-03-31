use super::data::*;
use crate::core::WithTracing;
use crate::openid::{TokenInjector, TokenProvider};
use crate::util::Client as ClientTrait;
use crate::{error::ClientError, Translator};
use futures::{stream, StreamExt, TryStreamExt};
use std::fmt::Debug;
use tracing::instrument;
use url::Url;

/// A device registry client
#[derive(Clone, Debug)]
pub struct Client<TP>
where
    TP: TokenProvider,
{
    client: reqwest::Client,
    registry_url: Url,
    token_provider: TP,
}

type ClientResult<T> = Result<T, ClientError<reqwest::Error>>;

impl<TP> ClientTrait<TP> for Client<TP>
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
    pub fn new(client: reqwest::Client, registry_url: Url, token_provider: TP) -> Self {
        Self {
            client,
            registry_url,
            token_provider,
        }
    }

    /// craft url for the registry
    fn url(&self, application: Option<&str>, device: Option<&str>) -> ClientResult<Url> {
        let mut url = self.registry_url.clone();

        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| ClientError::Request("Failed to get paths".into()))?;

            path.extend(&["api", "registry", "v1alpha1", "apps"]);
            if let Some(application) = application {
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
        }

        Ok(url)
    }

    /// List applications.
    ///
    /// Optionally pass a list of labels selectors to filter the list.
    ///
    /// If no applications exists, this function will return an empty Vec, otherwise it will return
    /// a list of applications.
    ///
    /// If the user does not have access to the API, the server side may return "not found"
    /// as a response instead of "forbidden".
    #[instrument]
    pub async fn list_apps<L>(&self, labels: Option<L>) -> ClientResult<Option<Vec<Application>>>
    where
        L: IntoIterator + Debug,
        L::Item: AsRef<str>,
    {
        let mut req = self.client().get(self.url(None, None)?);

        // todo it would be cool to have a programmatic way to construct labels selectors
        // using drogue-cloud-service-api::labels::LabelSelector
        // Also, allocating strings from the `&str` we have is terrible,
        // but using only `as_ref()` from the iter was dropping the reference after the loop.
        if let Some(labels) = labels {
            let label_string = labels
                .into_iter()
                .map(|item| item.as_ref().to_string())
                .collect::<Vec<String>>()
                .join(",");

            req = req.query(&[("labels", label_string)]);
        }

        let req = req
            .propagate_current_context()
            .inject_token(self.token_provider())
            .await?;

        Self::read_response(req.send().await?).await
    }

    /// Get an application by name.
    ///
    /// If the application do not exist, this function will return `None`, otherwise
    /// if will return the device information.
    ///
    /// If the user does not have access to the application, the server side may return "not found"
    /// as a response instead of "forbidden".
    #[instrument]
    pub async fn get_app<A>(&self, application: A) -> ClientResult<Option<Application>>
    where
        A: AsRef<str> + Debug,
    {
        self.read(self.url(Some(application.as_ref()), None)?).await
    }

    /// Get a device by name.
    ///
    /// If the application or device do not exist, this function will return `None`, otherwise
    /// if will return the device information.
    ///
    /// If the user does not have access to the application, the server side may return "not found"
    /// as a response instead of "forbidden".
    #[instrument]
    pub async fn get_device<A, D>(&self, application: A, device: D) -> ClientResult<Option<Device>>
    where
        A: AsRef<str> + Debug,
        D: AsRef<str> + Debug,
    {
        self.read(self.url(Some(application.as_ref()), Some(device.as_ref()))?)
            .await
    }

    /// Get a list of devices.
    ///
    /// The function will only return devices that could be found.
    #[instrument]
    pub async fn get_devices<A, D>(
        &self,
        application: A,
        devices: &[D],
    ) -> ClientResult<Vec<Device>>
    where
        A: AsRef<str> + Debug,
        D: AsRef<str> + Debug,
    {
        stream::iter(devices)
            .then(|device| self.get_device(application.as_ref(), device))
            // filter out missing devices
            .filter_map(|device| async { device.transpose() })
            // collect to a map
            .try_collect()
            .await
    }

    /// Get a device by name, resolving all first level gateways.
    #[instrument]
    pub async fn get_device_and_gateways<A, D>(
        &self,
        application: A,
        device: D,
    ) -> ClientResult<Option<(Device, Vec<Device>)>>
    where
        A: AsRef<str> + Debug,
        D: AsRef<str> + Debug,
    {
        let device: Option<Device> = self
            .read(self.url(Some(application.as_ref()), Some(device.as_ref()))?)
            .await?;

        if let Some(device) = device {
            let gateways = if let Some(gw_sel) = device
                .section::<DeviceSpecGatewaySelector>()
                .and_then(|s| s.ok())
            {
                // lookup devices
                self.get_devices(application, &gw_sel.match_names).await?
            } else {
                // unable to process gateways or no gateways configured
                vec![]
            };

            Ok(Some((device, gateways)))
        } else {
            Ok(None)
        }
    }

    /// List devices.
    ///
    /// Optionally pass a list of labels selectors to filter the list.
    ///
    /// If no devices exists, this function will return an empty Vec, otherwise it will return
    /// a list of devices.
    ///
    /// If the user does not have access to the API, the server side may return "not found"
    /// as a response instead of "forbidden".
    #[instrument]
    pub async fn list_devices<A, L>(
        &self,
        application: A,
        labels: Option<L>,
    ) -> ClientResult<Option<Vec<Device>>>
    where
        A: AsRef<str> + Debug,
        L: IntoIterator + Debug,
        L::Item: AsRef<str>,
    {
        let mut req = self
            .client()
            .get(self.url(Some(application.as_ref()), Some(""))?);

        // todo refactor this duplicated code
        if let Some(labels) = labels {
            let label_string = labels
                .into_iter()
                .map(|item| item.as_ref().to_string())
                .collect::<Vec<String>>()
                .join(",");

            req = req.query(&[("labels", label_string)]);
        }

        let req = req
            .propagate_current_context()
            .inject_token(self.token_provider())
            .await?;

        Self::read_response(req.send().await?).await
    }

    /// Update (overwrite) an application.
    ///
    /// The application must exist, otherwise `false` is returned.
    #[instrument]
    pub async fn update_app(&self, application: &Application) -> ClientResult<bool> {
        self.update(
            self.url(Some(application.metadata.name.as_str()), None)?,
            Some(application),
        )
        .await
    }

    /// Update (overwrite) a device.
    ///
    /// The application and device must exist, otherwise `false` is returned.
    #[instrument]
    pub async fn update_device(&self, device: &Device) -> ClientResult<bool> {
        self.update(
            self.url(
                Some(device.metadata.application.as_str()),
                Some(device.metadata.name.as_str()),
            )?,
            Some(device),
        )
        .await
    }

    /// Create a new application.
    #[instrument]
    pub async fn create_app(&self, app: &Application) -> ClientResult<Option<()>> {
        self.create(self.url(None, None)?, Some(app)).await
    }

    #[instrument]
    pub async fn delete_app<A>(&self, application: A) -> ClientResult<bool>
    where
        A: AsRef<str> + Debug,
    {
        self.delete(self.url(Some(application.as_ref()), None)?)
            .await
    }

    /// Create a new device.
    #[instrument]
    pub async fn create_device(&self, device: &Device) -> ClientResult<Option<()>> {
        self.create(
            self.url(Some(device.metadata.application.as_str()), Some(""))?,
            Some(device),
        )
        .await
    }

    #[instrument]
    pub async fn delete_device<A, D>(&self, application: A, device: D) -> ClientResult<bool>
    where
        A: AsRef<str> + Debug,
        D: AsRef<str> + Debug,
    {
        self.delete(self.url(Some(application.as_ref()), Some(device.as_ref()))?)
            .await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::openid::NoTokenProvider;

    #[test]
    fn test_url_list() -> anyhow::Result<()> {
        let client = Client::new(
            Default::default(),
            Url::parse("http://localhost")?,
            NoTokenProvider,
        );

        let url = client.url(None, None).unwrap();
        assert_eq!(
            url.to_string(),
            "http://localhost/api/registry/v1alpha1/apps",
        );

        Ok(())
    }

    #[test]
    fn test_url_app() -> anyhow::Result<()> {
        let client = Client::new(
            Default::default(),
            Url::parse("http://localhost")?,
            NoTokenProvider,
        );

        let url = client.url(Some("foo"), Some("bar/baz")).unwrap();
        assert_eq!(
            url.to_string(),
            "http://localhost/api/registry/v1alpha1/apps/foo/devices/bar%2Fbaz"
        );

        Ok(())
    }
}
