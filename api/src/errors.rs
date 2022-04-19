use axum::{
    http::StatusCode,
    Json,
    response::{IntoResponse, Response},
};
use serde_json::{json, Value};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;
pub type ApiError = (StatusCode, Json<Value>);
pub type ApiResult<T> = std::result::Result<T, ApiError>;

#[derive(Error)]
pub enum Error {
    #[error("{0}")]
    ValidationError(String),

    #[error("Record not found.")]
    NotFoundError(sqlx::Error),

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
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => Self::NotFoundError(e),
            sqlx::Error::Database(dbe) if dbe.to_string().contains("duplicate key value") => {
                Self::DuplicateResource
            }
            _ => Self::UnexpectedError(anyhow::Error::from(e)),
        }
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(e: argon2::password_hash::Error) -> Self {
        Self::InvalidAuthentication(anyhow::Error::msg(e.to_string()))
    }
}


impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status_code = match self {
            Error::ValidationError(_) => StatusCode::BAD_REQUEST,
            Error::NotFoundError(_) => StatusCode::NOT_FOUND,
            Error::DuplicateResource => StatusCode::CONFLICT,
            Error::InvalidAuthentication(_) => StatusCode::UNAUTHORIZED,
            Error::InsufficientPermissionsError => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR
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
