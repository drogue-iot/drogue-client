use core::fmt::{Display, Formatter};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransferOwnership {
    pub new_user: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Members {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_version: Option<String>,
    #[serde(default)]
    pub members: IndexMap<String, MemberEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MemberEntry {
    pub roles: Vec<Role>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    /// Allow editing app members and delete the app.
    Admin,
    /// Allow reading and writing devices and application details.
    Manager,
    /// Allow reading only of app and devices details.
    Reader,
    /// Allow consuming app events.
    Subscriber,
    /// Allow publishing command to the apps
    Publisher,    // publish commands to app
    /// grants all then permissions.
    Owner,
}

impl Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin => write!(f, "Administrator"),
            Self::Manager => write!(f, "Manager"),
            Self::Reader => write!(f, "Reader"),
            Self::Subscriber => write!(f, "Subscriber"),
            Self::Publisher => write!(f, "Publisher"),
            Self::Owner => write!(f, "Owner"),
        }
    }
}

impl FromStr for Role {
    type Err = ();

    fn from_str(input: &str) -> Result<Role, Self::Err> {
        match input {
            "Admin" | "admin" => Ok(Role::Admin),
            "Manager" | "manager" => Ok(Role::Manager),
            "Reader" | "reader" => Ok(Role::Reader),
            "Subscriber" | "subscriber" => Ok(Role::Subscriber),
            "Publisher" | "publisher" => Ok(Role::Publisher),
            "Owner" | "owner" => Ok(Role::Owner),
            _ => Err(()),
        }
    }
}
