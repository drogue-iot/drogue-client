//! A client for the Drogue IoT Cloud APIs.

pub mod error;
pub mod meta;
#[cfg(feature = "openid")]
pub mod openid;
pub mod registry;

mod context;
mod serde;
mod translator;

pub use context::*;
pub use translator::*;
