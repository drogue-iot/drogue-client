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
    pub roles: Roles,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Roles(pub Vec<Role>);

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    /// All the permissions except owning the app.
    /// Edit members for the applications.
    /// Read and write details (devices as well).
    /// Publish and subscribe.
    Admin,
    /// Allow reading and writing devices and application details.
    Manager,
    /// Allow reading only of app and devices details.
    Reader,
    /// Allow consuming app events.
    Subscriber,
    /// Allow publishing command to the apps
    Publisher,
}

impl Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin => write!(f, "Administrator"),
            Self::Manager => write!(f, "Manager"),
            Self::Reader => write!(f, "Reader"),
            Self::Subscriber => write!(f, "Subscriber"),
            Self::Publisher => write!(f, "Publisher"),
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
            _ => Err(()),
        }
    }
}

impl Roles {
    pub fn contains(&self, role: &Role) -> bool {
        if self.0.contains(&Role::Admin) {
            return true
        } else {
            match role {
                Role::Admin => self.0.contains(&Role::Admin),
                Role::Manager => self.0.contains(&Role::Manager),
                Role::Reader => self.0.contains(&Role::Manager) || self.0.contains(&Role::Reader),
                Role::Publisher => self.0.contains(&Role::Publisher),
                Role::Subscriber => self.0.contains(&Role::Subscriber),
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_admin_include_all_roles() {

        let roles = Roles(vec![Role::Admin]);

        assert!(roles.contains(&Role::Publisher));
        assert!(roles.contains(&Role::Subscriber));
        assert!(roles.contains(&Role::Reader));
        assert!(roles.contains(&Role::Manager));
        assert!(roles.contains(&Role::Admin));
    }

    #[test]
    fn role_manager_include_reader() {

        let roles = Roles(vec![Role::Manager]);

        assert!(roles.contains(&Role::Reader));
        assert!(roles.contains(&Role::Manager));
        assert!(roles.contains(&Role::Manager));
    }


    #[test]
    fn role_reader_include_reader() {

        let roles = Roles(vec![Role::Reader]);

        assert!(roles.contains(&Role::Reader));
        assert_eq!(roles.contains(&Role::Manager), false);
        assert_eq!(roles.contains(&Role::Admin), false);
    }

    #[test]
    fn roles_publisher() {

        let roles = Roles(vec![Role::Publisher]);

        assert!(roles.contains(&Role::Publisher));
        assert_eq!(roles.contains(&Role::Manager), false);
        assert_eq!(roles.contains(&Role::Admin), false);
        assert_eq!(roles.contains(&Role::Reader), false);
        assert_eq!(roles.contains(&Role::Subscriber), false);
    }

    #[test]
    fn roles_subscriber() {

        let roles = Roles(vec![Role::Subscriber]);

        assert!(roles.contains(&Role::Subscriber));
        assert_eq!(roles.contains(&Role::Manager), false);
        assert_eq!(roles.contains(&Role::Admin), false);
        assert_eq!(roles.contains(&Role::Reader), false);
        assert_eq!(roles.contains(&Role::Publisher), false);
    }
}