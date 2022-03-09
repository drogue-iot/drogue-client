use crate::registry::v1::ExternalEndpoint;
use crate::{core, dialect, serde::is_default, Section};
use serde::{Deserialize, Serialize};

dialect!(KnativeAppSpec [Section::Spec => "knative"]);

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct KnativeAppSpec {
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub disabled: bool,
    pub endpoint: ExternalEndpoint,
}

dialect!(KnativeAppStatus [Section::Status => "knative"]);

#[derive(Clone, Default, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct KnativeAppStatus {
    pub observed_generation: u64,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub conditions: core::v1::Conditions,
}
