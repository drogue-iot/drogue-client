use crate::{core, dialect, Dialect, Section};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KafkaAppStatus {
    pub observed_generation: u64,
    pub conditions: core::v1::Conditions,
    /// An explicit Kafka downstream status.
    ///
    /// This may be provided by the system to direct the downstream events (device-to-cloud) to an
    /// alternate Kafka target. If provided, this must contain both the server as well as the topic.
    /// Most likely, access credentials have to be provided too.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downstream: Option<KafkaDownstreamStatus>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KafkaDownstreamStatus {
    pub topic: String,
    pub bootstrap_servers: String,
    pub properties: HashMap<String, String>,
}

dialect!(KafkaAppStatus[Section::Status => "kafka"]);
