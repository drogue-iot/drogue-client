//! Version v1

/// Client-initiated functionality
pub mod client {

    /// Protocol messages, sent by the client
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub enum Message {
        /// Set a new access token.
        ///
        /// If the access token is validated, then it will replace the current access token in the
        /// session. If the token could not be validated, the server will close the connection.
        RefreshAccessToken(String),
    }
}
