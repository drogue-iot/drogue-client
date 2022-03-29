use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct AccessToken {
    /// The creation date of the access token
    pub created: DateTime<Utc>,
    /// The access token prefix
    pub prefix: String,
    /// The access token description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct CreatedAccessToken {
    /// The complete access token
    #[serde(default)]
    pub token: String,
    /// The access token prefix
    #[serde(default)]
    pub prefix: String,
}
