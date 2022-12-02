use crate::admin::v1::Role;
use crate::user::v1::authz::TokenPermission;
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct AccessTokenScopes {
    /// Allow creating applications
    pub create: bool,
    /// Scopes are defined for each application
    pub applications: IndexMap<String, Vec<Role>>,
    /// Access Tokens permissions
    pub tokens: Vec<TokenPermission>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct AccessToken {
    /// The creation date of the access token
    pub created: DateTime<Utc>,
    /// The access token prefix
    pub prefix: String,
    /// The access token description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub scopes: Option<AccessTokenScopes>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessTokenCreationOptions {
    pub description: Option<String>,
    /// If no scopes are provided, the access token
    /// will have the same permissions as its owner
    pub scopes: Option<AccessTokenScopes>,
}
