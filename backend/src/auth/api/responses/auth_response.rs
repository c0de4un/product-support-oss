use serde::Serialize;
use crate::user::entities::user::User;

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}