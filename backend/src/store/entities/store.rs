use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Store {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub domain: Option<String>,
    pub description: Option<String>,
    pub api_token: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}