//! Structures to work with users and identities.

use crate::tokens::v1::AccessTokenClaims;
use serde::{Deserialize, Serialize};

/// Details on an authenticated user.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserDetails {
    /// A unique user ID.
    pub user_id: String,
    /// Granted roles.
    pub roles: Vec<String>,
    /// Limited Authorization claims.
    /// This will be Some only in the context of a PAT authentication
    pub claims: Option<AccessTokenClaims>,
}
