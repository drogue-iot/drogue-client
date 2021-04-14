mod provider;

pub use provider::*;

#[cfg(feature = "reqwest")]
use crate::{context::Context, error::ClientError};
use chrono::{DateTime, Utc};

pub trait Expires {
    /// Check if the resources expires before the duration elapsed.
    fn expires_before(&self, duration: chrono::Duration) -> bool {
        match self.expires_in() {
            Some(expires) => expires <= duration,
            None => false,
        }
    }

    /// Get the duration until this resource expires. This may be negative.
    fn expires_in(&self) -> Option<chrono::Duration> {
        self.expires().map(|expires| expires - chrono::Utc::now())
    }

    /// Get the timestamp when the resource expires.
    fn expires(&self) -> Option<chrono::DateTime<chrono::Utc>>;
}

impl Expires for openid::Bearer {
    fn expires(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.expires
    }
}

impl Expires for DateTime<Utc> {
    fn expires(&self) -> Option<DateTime<Utc>> {
        Some(*self)
    }
}

#[cfg(test)]
mod test {
    use crate::openid::Expires;
    use chrono::*;

    #[test]
    fn test_expires_before() {
        let now = Utc::now();
        let timeout = now + Duration::seconds(30);

        assert!(!timeout.expires_before(Duration::seconds(10)));
        assert!(timeout.expires_before(Duration::seconds(60)));
    }
}

#[cfg(feature = "reqwest")]
pub(crate) async fn inject_token(
    token_provider: Option<OpenIdTokenProvider>,
    builder: reqwest::RequestBuilder,
    mut context: Context,
) -> Result<reqwest::RequestBuilder, ClientError<reqwest::Error>> {
    if let Some(token) = context.provided_token.take() {
        Ok(builder.bearer_auth(token))
    } else if let Some(provider) = token_provider {
        let token = provider
            .provide_token()
            .await
            .map_err(|err| ClientError::Token(Box::new(err)))?;
        Ok(builder.bearer_auth(token.access_token))
    } else {
        Ok(builder)
    }
}
