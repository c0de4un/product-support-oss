use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChatMessage {
    pub id: String,
    pub room_id: String,
    pub role: String,
    pub content: String,
    pub sources: Option<String>,
    pub model: Option<String>,
    pub latency_ms: Option<i32>,
    pub cached: bool,
    pub error_message: Option<String>,
    pub created_at: String,
}