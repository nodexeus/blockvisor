use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Wrapper Error enum used to provide a consistent [`IntoResponse`] target for
/// request handlers that return inner domain Error types.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    ValidationError(String),

    #[error("Record not found.")]
    NotFoundError,

    #[error("Duplicate resource conflict.")]
    DuplicateResource,

    #[error("invalid authentication credentials")]
    InvalidAuthentication(anyhow::Error),

    #[error("Insufficient permission.")]
    InsufficientPermissionsError,

    #[error("Error processing JWT")]
    JWTError(#[from] jsonwebtoken::errors::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),

    #[error("Database error")]
    SqlError(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::ValidationError(s) => (StatusCode::BAD_REQUEST, s),
            AppError::NotFoundError => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::DuplicateResource => (StatusCode::CONFLICT, self.to_string()),
            AppError::InvalidAuthentication(_e) => {
                (StatusCode::UNAUTHORIZED, "Unauthorized".into())
            }
            AppError::InsufficientPermissionsError => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::JWTError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::UnexpectedError(_e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".into(),
            ),
            AppError::SqlError(e) => match e {
                sqlx::Error::RowNotFound => {
                    (StatusCode::NOT_FOUND, AppError::NotFoundError.to_string())
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            },
        };
        let body = Json(json!({ "error": message }));

        (status, body).into_response()
    }
}
