use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[cfg(feature = "vsock")]
    /// Unknown method
    #[error("Method `{method}` not found")]
    UnknownMethod { method: String },
    #[cfg(feature = "vsock")]
    /// Network call method is in config but no host specified
    #[error("No host specified for method `{method}`")]
    NoHostSpecified { method: String },
    #[cfg(feature = "vsock")]
    /// Error while running sh command
    #[error("Failed to run command `{args}`, got output `{output}`")]
    Command { args: String, output: String },

    /// Reqwest error
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Error {
    #[cfg(feature = "vsock")]
    pub fn unknown_method(method: impl std::fmt::Display) -> Self {
        Self::UnknownMethod {
            method: method.to_string(),
        }
    }

    #[cfg(feature = "vsock")]
    pub fn no_host(method: impl std::fmt::Display) -> Self {
        Self::NoHostSpecified {
            method: method.to_string(),
        }
    }

    #[cfg(feature = "vsock")]
    pub fn command(args: impl std::fmt::Display, out: impl std::fmt::Debug) -> Self {
        Self::Command {
            args: args.to_string(),
            output: format!("{out:?}"),
        }
    }
}
