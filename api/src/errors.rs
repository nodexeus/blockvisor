use axum::{
    http::StatusCode,
    Json,
    response::{IntoResponse, Response},

use serde_json::json;

/// Wrapper Error enum used to provide a consistent [`IntoResponse`] target for
/// request handlers that return inner domain Error types.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("database error")]
    SqlError(#[from] sqlx::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::SqlError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error"),

        };
        let payload = json!({"message": self.to_string()});
        (status_code, payload.to_string()).into_response()
    }
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}


impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        let status = match err {
            Error::ValidationError(_) => StatusCode::BAD_REQUEST,
            Error::NotFoundError(_) => StatusCode::NOT_FOUND,
            Error::DuplicateResource => StatusCode::CONFLICT,
            Error::InvalidAuthentication(_) => StatusCode::UNAUTHORIZED,
            Error::InsufficientPermissionsError => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        };
        let payload = json!({"message": err.to_string()});
        (status, Json(payload))
    }
}
