use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// Unknown method
    #[error("Method `{method}` not found")]
    UnknownMethod { method: String },

    /// Network call method is in config but no host specified
    #[error("No host specified for method `{method}`")]
    NoHostSpecified { method: String },

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
