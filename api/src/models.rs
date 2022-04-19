use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use chrono::{DateTime, Utc};
use std::fmt;
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, rand_core::OsRng, SaltString},
};
use crate::errors::{Error, Result};
use anyhow::anyhow;
use crate::auth;
use sendgrid::v3::{
    Content,
    Email,
    Message,
    Personalization,
    Sender};
use sqlx::PgPool;


#[derive(Debug, Serialize, Deserialize,Validate)]
pub struct RegistrationReq {
    pub first_name: String,
    pub last_name: String,
    #[validate(email)]
    pub email: String,
    pub organization: Option<String>,
    #[validate(length(min = 8), must_match = "password_confirm")]
    pub password: String,
    pub password_confirm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub hashword: String,
    pub role: UserRole,
    #[serde(skip_serializing)]
    pub salt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserSummary {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub organization: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UserLoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRefreshRequest {
    pub refresh: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PasswordResetRequest {
    #[validate(email)]
    pub email: String,
}


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PwdResetInfo {
    pub token: String,
    #[validate(length(min = 8), must_match = "password_confirm")]
    pub password: String,
    pub password_confirm: String,
}

impl User {
    pub async fn create_user(user: RegistrationReq, db_pool: &PgPool) -> Result<User> {
        let _ = user
            .validate()
            .map_err(|e| Error::ValidationError(e.to_string()))?;

        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        if let Some(hashword) = argon2
            .hash_password(user.password.as_bytes(), salt.as_str())?
            .hash
        {
            let user = sqlx::query_as::<_, User>(
                r#"
                INSERT INTO
                   users (email, hashword, salt,first_name,last_name)
                values
                   (
                      $1::TEXT::CITEXT, $2, $3, $4, $5
                   )
                   RETURNING *
                "#,
            )
                .bind(user.email.as_str())
                .bind(hashword.to_string())
                .bind(salt.as_str())
                .bind(user.first_name)
                .bind(user.last_name)
                .fetch_one(db_pool)              
                .await?
                .set_jwt()
                .map_err(Error::from)?;
                
            Ok(user)
        } else {
            Err(Error::ValidationError("Invalid password.".to_string()))
        }
    }

    pub async fn login(user_login_req: UserLoginRequest, db_pool: &PgPool) -> Result<User> {
        let user = User::find_by_email(&user_login_req.email, db_pool)
            .await?
            .set_jwt()
            .map_err(|_e| {
                Error::InvalidAuthentication(anyhow!("Email or password is invalid."))
            })?;
        let _ = user.verify_password(&user_login_req.password)?;
        Ok(user)
    }

    pub async fn find_by_email(email: &str, db_pool: &PgPool) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT *
                    FROM   users
                    WHERE  Lower(email) = Lower($1)
                    LIMIT  1
                    "#
        )
            .bind(email)
            .fetch_one(db_pool)
            .await?;
        Ok(user)
        // .map_err(ApiError::from)
    }

    pub async fn find_summary_by_user(user_id: &Uuid, db_pool: &PgPool) -> Result<UserSummary> {
        let user = sqlx::query_as::<_, UserSummary>(
            r#"
            SELECT 
                users.id, 
                email
            FROM
                users
            WHERE
                users.id = $1
            "#
        )
            .bind(user_id)
            .fetch_one(db_pool)
            .await?;

        Ok(user)
    }

    pub async fn find_by_id(id: Uuid, pool: &PgPool) -> Result<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 limit 1")
            .bind(id)
            .fetch_one(pool)
            .await?;
        Ok(user)
    }

    pub async fn refresh(req: UserRefreshRequest, pool: &PgPool) -> Result<User> {
        let mut user = User::find_by_refresh(&req.refresh, pool).await?;
        user.set_jwt();
        Ok(user)
    }
    pub async fn find_by_refresh(refresh: &str, pool: &PgPool) -> Result<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE refresh = $1 limit 1")
            .bind(refresh)
            .fetch_one(pool)
            .await?;
        Ok(user)
    }


    pub async fn email_reset_password(req: PasswordResetRequest, db_pool: &PgPool) -> Result<()> {
        let user = User::find_by_email(&req.email, db_pool).await?;

        let auth_data = auth::UserAuthData {
            user_id: user.id,
            user_role: user.role.to_string(),
        };

        let token = auth::create_temp_jwt(&auth_data)?;

        let p = Personalization::new(Email::new(&user.email));

        let subject = "Reset Password".to_string();
        let body = format!(
            r##"
            <h1>Password Reset</h1>
            <p>You have requested to reset your BlockJoy password.
            Please visit <a href="https://console.blockjoy.com/reset?t={token}">
            https://console.blockjoy.com/reset?t={token}</a>.</p><br /><br /><p>Thank You!</p>"##
        );
        let sendgrid_api_key = dotenv::var("SENDGRID_API_KEY").map_err(|_| {
            Error::UnexpectedError(anyhow!("Could not find SENDGRID_API_KEY in env."))
        })?;
        let sender = Sender::new(sendgrid_api_key);
        let m = Message::new(Email::new("BlockJoy <hello@blockjoy.com>"))
            .set_subject(&subject)
            .add_content(Content::new().set_content_type("text/html").set_value(body))
            .add_personalization(p);

        sender
            .send(&m)
            .await
            .map_err(|_| Error::UnexpectedError(anyhow!("Could not send email")))?;

        Ok(())
    }


    pub async fn reset_password(req: &PwdResetInfo, db_pool: &PgPool) -> Result<User> {
        let _ = req
            .validate()
            .map_err(|e| Error::ValidationError(e.to_string()))?;

        match auth::validate_jwt(&req.token)? {
            auth::JwtValidationStatus::Valid(auth_data) => {
                let user = User::find_by_id(auth_data.user_id, db_pool).await?;
                return User::update_password(user, &req.password, db_pool).await;
            }
            _ => Err(Error::InsufficientPermissionsError),
        }
    }


    pub async fn update_password(user: User, password: &str, pool: &PgPool) -> Result<User> {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        if let Some(hashword) = argon2
            .hash_password(password.as_bytes(), salt.as_str())?
            .hash
        {
            return sqlx::query_as::<_, User>(
                r#"
                UPDATE
                  users
                SET
                  hashword = $1,
                  salt = $2
                WHERE
                  id = $3 RETURNING *
                "#
            )
                .bind(hashword.to_string())
                .bind(salt.as_str())
                .bind(user.id)
                .fetch_one(pool)
                .await
                .map_err(Error::from)?
                .set_jwt();
        }

        Err(Error::ValidationError("Invalid password.".to_string()))
    }
    pub fn verify_password(&self, password: &str) -> Result<()> {
        let argon2 = Argon2::default();
        let parsed_hash = argon2.hash_password(password.as_bytes(), &self.salt)?;

        if let Some(output) = parsed_hash.hash {
            if self.hashword == output.to_string() {
                return Ok(());
            }
        }
        Err(Error::InvalidAuthentication(anyhow!("Invalid email or password.")))
    }

    pub fn set_jwt(&mut self) -> Result<Self> {
        let auth_data = auth::UserAuthData {
            user_id: self.id,
            user_role: self.role.to_string(),
        };
        self.token = Some(auth::create_jwt(&auth_data)?);
        Ok(self.to_owned())
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "enum_user_role", rename_all = "snake_case")]
pub enum UserRole {
    User,
    Admin,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Admin => write!(f, "admin"),
            Self::User => write!(f, "user"),
        }
    }
}
