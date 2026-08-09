use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct ChatRequest {
    pub store_id: Uuid,

    #[validate(length(min = 1, max = 2000, message = "Question must be between 1 and 2000 characters"))]
    pub question: String,

    pub room_id: Option<Uuid>,
}