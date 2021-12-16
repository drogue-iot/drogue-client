use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

fn epoch() -> DateTime<Utc> {
    Utc.timestamp_millis(0)
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct AccessToken {
    /// The creation date of the access token
    #[serde(default = "epoch")]
    pub created: DateTime<Utc>,
    /// The access token prefix
    #[serde(default)]
    pub prefix: String,
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
