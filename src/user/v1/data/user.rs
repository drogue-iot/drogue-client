//! Structures to work with users and identities.

use crate::tokens::v1::AccessTokenScopes;
use serde::{Deserialize, Serialize};

/// Details on an authenticated user.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserDetails {
    /// A unique user ID.
    pub user_id: String,
    /// Granted roles.
    pub roles: Vec<String>,
    /// Limited Authorization scopes.
    pub scopes: Option<AccessTokenScopes>,
}
