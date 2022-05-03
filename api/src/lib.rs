#![warn(unreachable_pub, unused_extern_crates)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

pub mod auth;
pub mod config;
pub(crate) mod db;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod result;
pub mod server;
pub mod telemetry;
#[cfg(test)]
mod test_helpers;
