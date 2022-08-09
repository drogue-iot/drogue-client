//! Metrics support for clients

#[cfg(feature = "telemetry")]
mod ext;
mod pass;

#[cfg(feature = "telemetry")]
pub use ext::*;
pub use pass::*;
