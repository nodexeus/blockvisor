#![warn(unreachable_pub, unused_extern_crates)]
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::perf,
    clippy::style,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

pub mod config;
pub(crate) mod db;
pub mod errors;
pub mod handlers;
pub mod server;
pub mod telemetry;
#[cfg(test)]
mod test_helpers;
