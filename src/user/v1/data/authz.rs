use crate::metrics::{AsPassFail, PassFail};
use core::fmt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    Device(DevicePermission),
    App(ApplicationPermission),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenPermission {
    Create,  // Create new tokens
    List,    // Read all tokens for this user
    Delete,  // Delete any token for this user
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevicePermission {
    Create, // Create a resource
    Delete, // delete a resource
    Write,  // Write resource details
    Read,   // read resource details
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplicationPermission {
    Create,    // Create a resource
    Delete,    // delete a resource
    Write,     // Write resource details
    Read,      // read resource details
    Transfer,  // Transfer app to another owner
    Subscribe, // consume app events
    Command,   // publish commands to app
    Members,   // Members operations : read and write
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Authorize a request for a user.
///
/// NOTE: The user_id and roles information must come from a trusted source, like
/// a validated token. The user service will not re-validate this information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthorizationRequest {
    pub application: String,
    pub permission: Permission,

    pub user_id: Option<String>,
    pub roles: Vec<String>,
}

/// The outcome of an authorization request
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Outcome {
    Allow,
    Deny,
}

impl Outcome {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow)
    }

    pub fn ensure<F, E>(&self, f: F) -> Result<(), E>
    where
        F: FnOnce() -> E,
    {
        match self.is_allowed() {
            true => Ok(()),
            false => Err(f()),
        }
    }
}

/// The result of an authorization request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthorizationResponse {
    /// The outcome, of the request.
    pub outcome: Outcome,
}

impl AsPassFail for AuthorizationResponse {
    fn as_pass_fail(&self) -> PassFail {
        match self.outcome {
            Outcome::Allow => PassFail::Pass,
            Outcome::Deny => PassFail::Fail,
        }
    }
}
