use crate::{
    meta::v1::NonScopedMetadata, serde::Base64Standard, translator, Dialect, Section, Translator,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// An application.
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
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

/// The application's trust-anchors.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ApplicationSpecTrustAnchors {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub anchors: Vec<ApplicationSpecTrustAnchorEntry>,
}

impl Dialect for ApplicationSpecTrustAnchors {
    fn key() -> &'static str {
        "trustAnchors"
    }

    fn section() -> Section {
        Section::Spec
    }
}

/// A single trust-anchor entry.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ApplicationSpecTrustAnchorEntry {
    #[serde(with = "Base64Standard")]
    pub certificate: Vec<u8>,
}

/// The status of the trust-anchors.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ApplicationStatusTrustAnchors {
    pub anchors: Vec<ApplicationStatusTrustAnchorEntry>,
}

impl Dialect for ApplicationStatusTrustAnchors {
    fn key() -> &'static str {
        "trustAnchors"
    }

    fn section() -> Section {
        Section::Status
    }
}

/// A single trust-anchor status.
#[derive(Clone, Debug, Deserialize, Serialize)]
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
