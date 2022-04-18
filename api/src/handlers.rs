// This will eventually be broken out into sub files in module

use crate::models::RegistrationReq;
use axum::{http::StatusCode, Extension, Json};
use sqlx::PgPool;
use tracing::instrument;

/// GET handler for health requests by an application platform
///
/// Intended for use in environments such as Amazon ECS or Kubernetes which want
/// to validate that the HTTP service is available for traffic, by returning a
/// 200 OK response with any content.
#[allow(clippy::unused_async)]
pub async fn health() -> &'static str {
    "OK"
}

#[instrument(skip(db))]
pub async fn registration_create(
    db: Extension<PgPool>,
    Json(payload): Json<RegistrationReq>,
) -> StatusCode {
    unimplemented!()
}
