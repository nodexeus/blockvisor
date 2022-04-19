use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::anyhow;
use chrono::Utc;
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use axum::http::Request;
use futures_util::future::{err, ok, Ready};
use std::borrow::Cow;
use log::{debug, warn};
use sqlx::PgPool;
use crate::models::User;
use crate::models::UserRole;
use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts,TypedHeader},
};
use crate::errors::{Error, ApiError};
use headers::{authorization::Bearer, Authorization};

const JWT_SECRET: &str = "?G'A$jNW<$6x(PdFP?4VdRvmotIV^^";

#[derive(Debug)]
pub struct UserAuthData {
    pub user_id: Uuid,
    pub user_role: String,
}
#[derive(Debug)]
pub enum JwtValidationStatus {
    Valid(UserAuthData),
    Expired(UserAuthData),
    Invalid,
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
    role: String,
    exp: usize,
}

/// Creates a jwt with default duration for expires.
pub fn create_jwt(data: &UserAuthData) -> crate::errors::Result<String> {
    let duration = chrono::Duration::days(1);
    create_jwt_with_duration(data, duration)
}

pub fn create_temp_jwt(data: &UserAuthData) -> crate::errors::Result<String> {
    let duration = chrono::Duration::seconds(60 * 12);
    create_jwt_with_duration(data, duration)
}

pub fn create_jwt_with_duration(data: &UserAuthData, duration: chrono::Duration) -> crate::errors::Result<String> {
    let exp = Utc::now()
        .checked_add_signed(duration)
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: data.user_id.to_string(),
        role: data.user_role.clone(),
        exp: exp as usize,
    };

    let header = Header::new(Algorithm::HS512);
    Ok(encode(
        &header,
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )?)
}

pub fn validate_jwt(jwt: &str) -> crate::errors::Result<JwtValidationStatus> {
    let validation = Validation {
        leeway: 60,
        validate_exp: false,
        algorithms: vec![Algorithm::HS512],
        ..Default::default()
    };

    let result = match decode::<Claims>(
        jwt,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &validation,
    ) {
        Ok(decoded) => {
            let user_id = Uuid::parse_str(&decoded.claims.sub)
                .map_err(|_| Error::from(anyhow!("Error parsing uuid from JWT sub")))?;
            let user_role = decoded.claims.role;

            let auth_data = UserAuthData { user_id, user_role };

            // Check Expiration
            let exp = decoded.claims.exp;
            let now = get_current_timestamp();
            if (exp as u64) < now - 60 {
                JwtValidationStatus::Expired(auth_data)
            } else {
                JwtValidationStatus::Valid(auth_data)
            }
        }

        Err(_) => JwtValidationStatus::Invalid,
    };

    Ok(result)
}

fn jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| JWT_SECRET.to_string())
}

fn get_current_timestamp() -> u64 {
    let start = SystemTime::now();
    start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq,Copy)]
pub struct UserAuthInfo {
    pub id: Uuid,
    pub role: UserRole,
}

pub type UserAuthToken = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Authentication {
    User(UserAuthInfo),
    Service(UserAuthToken),
    Fail(String),
}

impl Authentication {
    pub fn is_user(&self) -> bool {
        matches!(self, Self::User(_))
    }
    pub fn is_admin(&self) -> bool {
        matches!(self, Self::User(u) if u.role == UserRole::Admin)
    }

    pub fn is_service(&self) -> bool {
        matches!(self, Self::Service(_))
    }

    /// Returns an error if not an admin
    pub fn try_admin(&self) -> crate::errors::Result<bool> {
        if self.is_admin() {
            Ok(true)
        } else {
            Err(Error::InsufficientPermissionsError)
        }
    }

    /// Returns an error if not an admin
    pub fn try_service(&self) -> crate::errors::Result<bool> {
        if self.is_service() {
            Ok(true)
        } else {
            Err(Error::InsufficientPermissionsError)
        }
    }

    /// Returns an error if user doesn't have access
    pub fn try_user_access(&self, user_id: Uuid) -> crate::errors::Result<bool> {
        match self {
            Self::User(u) if u.id == user_id => Ok(true),
            _ => Err(Error::InsufficientPermissionsError),
        }
    }

    pub async fn get_user(&self, pool: PgPool) -> crate::errors::Result<User> {
        match self {
            Self::User(u) => User::find_by_id(u.id, &pool).await,
            _ => Err(anyhow!("Authentication is not a user.").into()),
        }
    }
}
use std::str::FromStr;
impl FromStr for UserRole {
    type Err = ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "owner" => Ok(Self::User),
            _ => Ok(Self::Admin),
        }
    }
}
#[async_trait]
impl<B> FromRequest<B> for Authentication 
where
    B: Send,
{
    type Rejection = ApiError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        if let Some(token) = req
            .headers()
            .get("Authorization")
            .and_then(|hv| hv.to_str().ok())
            .and_then(|hv| {
                let words = hv.split("Bearer").collect::<Vec<&str>>();
                let token = words.get(1).map(|w| w.trim());
                token.map(Cow::Borrowed)
            })
        {
            let api_service_secret =
                std::env::var("API_SERVICE_SECRET").unwrap_or_else(|_| "".into());
            let is_service_token = !api_service_secret.is_empty() && token == api_service_secret;
            if token.starts_with("eyJ") {
                debug!("JWT Auth in Bearer.");
                if let Ok(JwtValidationStatus::Valid(auth_data)) =
                    validate_jwt(token.as_ref())
                {
                    if let Ok(role) = UserRole::from_str(&auth_data.user_role) {
                        return Ok(Self::User(UserAuthInfo {
                            id: auth_data.user_id,
                            role,
                        }));
                    }
                }
            } else if is_service_token {
                debug!("Service Auth in Bearer.");
                return Ok(Self::Service(token.as_ref().to_string()));
            }
        };

        warn!(
            "Invalid token auth: {:?} ",//- {:?}",
            req.headers().get("Authorization"),
            //req.path()
        );
        Ok(Self::Fail("invalid authentication credentials".to_string()))
    }

}
