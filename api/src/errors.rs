use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Wrapper Error enum used to provide a consistent [`IntoResponse`] target for
/// request handlers that return inner domain Error types.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error")]
    SqlError(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::SqlError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error"),
        };

        let body = Json(json!({ "error": message }));

        (status, body).into_response()
    }
}
