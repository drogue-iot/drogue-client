//! Core data structures.

#[cfg(feature = "reqwest")]
mod client;
pub mod v1;

#[cfg(feature = "reqwest")]
pub use client::*;
