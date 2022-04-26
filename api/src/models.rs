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
use sqlx::{PgPool,FromRow,Type,PgConnection};


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

#[derive(Debug, Clone, Serialize, Deserialize,FromRow)]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub orgs: Option<Vec<Org>>,
    #[serde(skip_serializing)]
    pub hashword: String,
    #[serde(skip_serializing)]
    pub salt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "enum_role", rename_all = "snake_case")]
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSummary {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
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


#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "enum_org_role", rename_all = "snake_case")]
pub enum UserOrgRole {
    Admin,
    Owner,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type,FromRow)]
pub struct Org {
    pub id: Uuid,
    pub name: String,
    pub is_personal: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<UserOrgRole>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub async fn create_user(user: RegistrationReq, db_pool: &PgPool) -> Result<Self> {
        let _ = user
            .validate()
            .map_err(|e| Error::ValidationError(e.to_string()));

        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        if let Some(hashword) = argon2
            .hash_password(user.password.as_bytes(), salt.as_str())?
            .hash
        {
            let mut tx = db_pool.begin().await?;
            let result = sqlx::query!(
                r#"
                INSERT INTO
                users (email, hashword, salt,first_name,last_name)
                values
                (
                    Lower($1), $2, $3, $4, $5
                )
                   RETURNING *
                "#,
                user.email,
                hashword.to_string(),
                salt.as_str(),
                user.first_name,
                user.last_name
            )
                .fetch_one(&mut tx)
                .await
                .map(|rec| User {
                    first_name: rec.first_name,
                    last_name: rec.last_name,
                    email:rec.email,
                    id:rec.id,
                    hashword:rec.hashword,
                    refresh:rec.refresh,
                    token:rec.token,
                    created_at:rec.created_at,
                    updated_at:rec.updated_at,
                    salt:rec.salt,
                    orgs:None

                })              
                .map_err(Error::from);

                // let mut user  =result.unwrap();
                // user.orgs = Some(Org::find_all_by_user(user.id,&mut tx).await?);

                tx.commit().await?; 

               
            Ok(result.unwrap())
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
        let mut tx = db_pool.begin().await?;
        let user = sqlx::query!(
            r#"SELECT *
                    FROM   users
                    WHERE  Lower(email) = Lower($1)
                    LIMIT  1
                    "#,
                    email
        )
            .fetch_one(&mut tx)
            .await
                .map(|rec| User {
                    first_name: rec.first_name,
                    last_name: rec.last_name,
                    email:rec.email,
                    id:rec.id,
                    hashword:rec.hashword,
                    refresh:rec.refresh,
                    token:rec.token,
                    created_at:rec.created_at,
                    updated_at:rec.updated_at,
                    salt:rec.salt,
                    orgs:None

                }).map_err(Error::from);    
                let mut user  =user.unwrap();
                user.orgs = Some(Org::find_all_by_user(user.id,&mut tx).await?);
                tx.commit().await?; 
        user.set_jwt()
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
        let user = sqlx::query!(r#"SELECT * FROM users WHERE id = $1 limit 1"#,id)
            .fetch_one(pool)
            .await
            .map(|rec| User {
                first_name: rec.first_name,
                last_name: rec.last_name,
                email:rec.email,
                id:rec.id,
                hashword:rec.hashword,
                refresh:rec.refresh,
                token:rec.token,
                created_at:rec.created_at,
                updated_at:rec.updated_at,
                salt:rec.salt,
                orgs:None

            }).map_err(Error::from);    
        Ok(user.unwrap())
    }

    pub async fn refresh(req: UserRefreshRequest, pool: &PgPool) -> Result<User> {
        let  user = User::find_by_refresh(&req.refresh, pool).await?.set_jwt().map_err(Error::from)?;
        Ok(user)
    }
    pub async fn find_by_refresh(refresh: &str, pool: &PgPool) -> Result<User> {
        let user = sqlx::query!(r#"SELECT * FROM users WHERE refresh = $1 limit 1"#, refresh)
            .fetch_one(pool)
            .await
            .map(|rec| User {
                first_name: rec.first_name,
                last_name: rec.last_name,
                email:rec.email,
                id:rec.id,
                hashword:rec.hashword,
                refresh:rec.refresh,
                token:rec.token,
                created_at:rec.created_at,
                updated_at:rec.updated_at,
                salt:rec.salt,
                orgs:None

            }).map_err(Error::from);    
        Ok(user.unwrap())
    }


    pub async fn email_reset_password(req: PasswordResetRequest, db_pool: &PgPool) -> Result<()> {
        let user = User::find_by_email(&req.email, db_pool).await?;

        let auth_data = auth::UserAuthData {
            user_id: user.id,
            user_role: user.first_name.to_string(),
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
            return sqlx::query!(
                r#"
                UPDATE
                users
                SET
                hashword = $1,
                salt = $2
                WHERE
                  id = $3 RETURNING *
                "#,
                hashword.to_string(),
                salt.as_str(),
                user.id
            )
                .fetch_one(pool).await
                .map(|rec| User {
                    first_name: rec.first_name,
                    last_name: rec.last_name,
                    email:rec.email,
                    id:rec.id,
                    hashword:rec.hashword,
                    refresh:rec.refresh,
                    token:rec.token,
                    created_at:rec.created_at,
                    updated_at:rec.updated_at,
                    salt:rec.salt,
                    orgs:None
    
                }).map_err(Error::from).unwrap().set_jwt(); 
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
            user_role: self.first_name.to_string(),
        };
        self.token = Some(auth::create_jwt(&auth_data)?);
        Ok(self.to_owned())
    }
}

impl Org{
    pub async fn find_by_name(name: &str, db_pool: &PgPool) -> Result<Org> {
        let org = sqlx::query!(
            r#"SELECT *
                    FROM   orgs
                    WHERE  Lower(name) = Lower($1)
                    LIMIT  1
                    "#,
                    name
        )
            .fetch_one(db_pool)
            .await
                .map(|rec| Org {
                    id:rec.id,
                    name:rec.name,
                    is_personal:rec.is_personal,
                    created_at:rec.created_at,
                    updated_at:rec.updated_at,
                    role:None
                }).map_err(Error::from);    
                Ok(org.unwrap())
    }

    pub async fn find_all_by_user(user_id: Uuid,  tx: &mut PgConnection) -> Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT o.id, o.name, o.is_personal,o.created_at, o.updated_at , ou.role FROM orgs o inner join orgs_users ou on o.id = ou.orgs_id WHERE users_id = $1 order by created_at DESC",
        )
        .bind(user_id)
        .fetch_all(tx)
        .await
        .map_err(Error::from)
    }
}


