mod downstream;
mod kafka;
mod knative;
mod process;

pub use downstream::*;
pub use kafka::*;
pub use knative::*;
pub use process::*;
use std::time::Duration;

use crate::{
    dialect,
    meta::v1::{CommonMetadata, CommonMetadataMut, NonScopedMetadata},
    serde::{is_default, Base64Standard},
    translator, Section, Translator,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// An application, owning devices.
#[derive(Clone, Default, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct Application {
    pub metadata: NonScopedMetadata,
    #[serde(default)]
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub spec: Map<String, Value>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub status: Map<String, Value>,
}

translator!(Application);

impl AsRef<dyn CommonMetadata> for Application {
    fn as_ref(&self) -> &(dyn CommonMetadata + 'static) {
        &self.metadata
    }
}

impl AsMut<dyn CommonMetadataMut> for Application {
    fn as_mut(&mut self) -> &mut (dyn CommonMetadataMut + 'static) {
        &mut self.metadata
    }
}

/// The application's trust-anchors.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct ApplicationSpecTrustAnchors {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub anchors: Vec<ApplicationSpecTrustAnchorEntry>,
}

dialect!(ApplicationSpecTrustAnchors [Section::Spec => "trustAnchors"]);

/// A single trust-anchor entry.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct ApplicationSpecTrustAnchorEntry {
    #[serde(with = "Base64Standard")]
    pub certificate: Vec<u8>,
}

/// The status of the trust-anchors.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct ApplicationStatusTrustAnchors {
    pub anchors: Vec<ApplicationStatusTrustAnchorEntry>,
}

dialect!(ApplicationStatusTrustAnchors [Section::Status => "trustAnchors"]);

/// A single trust-anchor status.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ApplicationStatusTrustAnchorEntry {
    #[serde(rename_all = "camelCase")]
    Valid {
        subject: String,
        #[serde(with = "Base64Standard")]
        certificate: Vec<u8>,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
    },
    Invalid {
        error: String,
        message: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalEndpoint {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    pub url: String,
    #[serde(default)]
    pub tls: Option<TlsOptions>,
    #[serde(default)]
    pub auth: Authentication,
    #[serde(default)]
    pub headers: Vec<Header>,
    #[serde(default)]
    #[serde(with = "humantime_serde")]
    pub timeout: Option<Duration>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
pub struct TlsOptions {
    #[serde(default)]
    pub insecure: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub certificate: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Authentication {
    None,
    Basic {
        username: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        password: Option<String>,
    },
    Bearer {
        token: String,
    },
}

impl Default for Authentication {
    fn default() -> Self {
        Self::None
    }
}
