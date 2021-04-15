/// A request context, used to add additional information for a request.
#[derive(Clone, Debug, Default)]
pub struct Context {
    /// A provided token to use, instead of a token from the client's token provider.
    pub provided_token: Option<String>,
}
