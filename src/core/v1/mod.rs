use crate::{Dialect, Section};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub last_transition_time: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default = "default_condition_status")]
    pub status: String,
    pub r#type: String,
}

fn default_condition_status() -> String {
    "Unknown".into()
}

impl Dialect for Vec<Condition> {
    fn key() -> &'static str {
        "conditions"
    }

    fn section() -> Section {
        Section::Status
    }
}
