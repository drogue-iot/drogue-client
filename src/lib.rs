//! A client for the Drogue IoT Cloud APIs.

pub mod admin;
pub mod core;
pub mod error;
pub mod meta;
#[cfg(feature = "openid")]
pub mod openid;
pub mod registry;
pub mod tokens;

mod serde;
mod translator;
mod util;

pub use translator::*;
