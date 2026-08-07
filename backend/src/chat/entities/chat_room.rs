use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChatRoom {
    pub id: String,
    pub user_id: String,
    pub store_id: Option<String>,
    pub title: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}