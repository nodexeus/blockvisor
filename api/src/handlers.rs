// This will eventually be broken out into sub files in module


use authy::{Client, Status};
use axum::{Extension, http::StatusCode, Json, response::{IntoResponse, Response}};

use crate::{
    errors::ApiError,
    models::{RegistrationReq, User},
};
use anyhow::Result;
use axum::{http::StatusCode, Extension, Json};

use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::auth::Authentication;
use crate::config::AuthyConfig;
use crate::models::{AuthyRegistrationReq, AuthyUser, AuthyVerifyReq, PasswordResetRequest};
use crate::models::PwdResetInfo;
use crate::models::RegistrationReq;
use crate::models::User;
use crate::models::UserLoginRequest;
use crate::models::UserRefreshRequest;

type ApiResponse = crate::errors::Result<Response>;

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
) -> Result<(StatusCode, Json<User>), ApiError> {
    unimplemented!()
}

pub async fn create_user(
    Json(user_req): Json<RegistrationReq>,
    Extension(db_pool): Extension<PgPool>,
) -> ApiResponse
{
    let result = User::create_user(user_req, &db_pool).await;
    match result {
        Ok(summary) => Ok((Json(summary)).into_response()),
        Err(err) => {
            Err(err)
        }
    }
}

pub async fn user_summary(
    Extension(db_pool): Extension<PgPool>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    auth: Authentication,
) -> ApiResponse
{
    let _ = auth.try_admin()?;
    let result = User::find_summary_by_user(&id, &db_pool).await;

    match result {
        Ok(summary) => Ok((Json(summary)).into_response()),
        Err(err) => {
            Err(err)
        }
    }
}


pub async fn login(
    Json(user_login_req): Json<UserLoginRequest>,
    Extension(db_pool): Extension<PgPool>,
) -> ApiResponse {
    let result = User::login(user_login_req, &db_pool).await;
    match result {
        Ok(user) => Ok((Json(user)).into_response()),
        Err(err) => {
            Err(err)
        }
    }
}

pub async fn whoami(Extension(db_pool): Extension<PgPool>,
                    //     auth: Authentication,
                    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> ApiResponse {
    let result = User::find_by_id(id, &db_pool).await;
    match result {
        Ok(user) => Ok((Json(user)).into_response()),
        Err(err) => {
            Err(err)
        }
    }
}

pub async fn refresh(Json(req): Json<UserRefreshRequest>, Extension(db_pool): Extension<PgPool>) -> ApiResponse {
    let result = User::refresh(req, &db_pool).await;
    match result {
        Ok(user) => Ok((Json(user)).into_response()),
        Err(err) => {
            Err(err)
        }
    }
}


pub async fn reset_pwd(
    Json(password_reset_req): Json<PasswordResetRequest>,
    Extension(db_pool): Extension<PgPool>,
) -> ApiResponse {
    let result = User::email_reset_password(password_reset_req, &db_pool).await;
    match result {
        Ok(_) => Ok((Json("An email with reset instructions has been sent.".to_string())).into_response()),
        Err(err) => {
            Err(err)
        }
    }
}


pub async fn update_pwd(
    Json(password_reset_req): Json<PwdResetInfo>,
    Extension(db_pool): Extension<PgPool>,
) -> ApiResponse {
    let result = User::reset_password(&password_reset_req, &db_pool).await;
    match result {
        Ok(user) => Ok((Json(user)).into_response()),
        Err(err) => {
            Err(err)
        }
    }
}

// TODO : move to config files
const API_URL: &str = "https://api.authy.com";
const API_KEY: &str = "yb0mhD2F2Otb5CJxRoPWLWnTNeQgGs5U";

pub async fn authy_register(
    Json(authy_reg_req): Json<AuthyRegistrationReq>,
    // Extension(db_pool): Extension<PgPool>,
) -> ApiResponse {
    let client = Client::new(API_URL, API_KEY);
    let result = AuthyUser::register(&client, &authy_reg_req).await;
    match result {
        Ok(user) => Ok((Json(user)).into_response()),
        Err(err) => {
            Err(err)
        }
    }
}

pub async fn authy_qr(
    // Json(authy_id_req): Json<AuthyIDReq>,
    // Extension(db_pool): Extension<PgPool>,
) -> ApiResponse {
    unimplemented!()
}

pub async fn authy_verify(
    Json(authy_verify_req): Json<AuthyVerifyReq>,
    // Extension(db_pool): Extension<PgPool>,
) -> ApiResponse {
    let client = Client::new(API_URL, API_KEY);
    let result = AuthyUser::verify(&client, &authy_verify_req).await;
    match result {
        Ok(verified) => Ok((Json(verified)).into_response()),
        Err(err) => {
            Err(err)
        }
    }
}
