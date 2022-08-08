//! A client for the Drogue IoT Cloud APIs.

pub mod admin;
pub mod command;
pub mod core;
pub mod discovery;
pub mod error;
pub mod integration;
pub mod meta;
pub mod metrics;
pub mod openid;
pub mod registry;
pub mod tokens;
pub mod user;

mod serde;
mod translator;

pub use translator::*;
