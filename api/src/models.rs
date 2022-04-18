use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationReq {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub organization: Option<String>,
    pub password: String,
    pub password_confirm: String,
}
