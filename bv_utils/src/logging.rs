use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "bv_fmt_log")]
use tracing_subscriber::{EnvFilter, Layer, Registry};

#[cfg(not(feature = "bv_fmt_log"))]
pub fn setup_logging() {
    if let Ok(journald) = tracing_journald::layer() {
        let _ = tracing_subscriber::registry().with(journald).try_init();
    }
}

#[cfg(feature = "bv_fmt_log")]
pub fn setup_logging() {
    let fmt = <tracing_subscriber::fmt::Layer<Registry> as Layer<Registry>>::with_filter(
        tracing_subscriber::fmt::layer(),
        EnvFilter::from_default_env(),
    );
    let _ = tracing_subscriber::registry().with(fmt).try_init();
}
