use std::sync::Arc;
use uuid::Uuid;

use crate::chat::entities::chat_message::ChatMessage;
use crate::chat::entities::chat_room::ChatRoom;
use crate::chat::repositories::chat_repository::ChatRepository;
use crate::llm::provider::{LlmError, LlmMessage, LlmProvider};
use crate::store::services::store_service::{StoreService, StoreServiceError};

#[derive(Debug)]
pub enum ChatServiceError {
    StoreNotFound,
    LlmError(String),
    InternalError(String),
}

impl From<StoreServiceError> for ChatServiceError {
    fn from(err: StoreServiceError) -> Self {
        match err {
            StoreServiceError::NotFound => ChatServiceError::StoreNotFound,
            _ => ChatServiceError::InternalError(err.to_string()),
        }
    }
}

impl From<LlmError> for ChatServiceError {
    fn from(err: LlmError) -> Self {
        ChatServiceError::LlmError(err.to_string())
    }
}

pub struct ChatAskResult {
    pub room: ChatRoom,
    pub user_message: ChatMessage,
    pub assistant_message: ChatMessage,
}

pub struct ChatService {
    chat_repo: Arc<ChatRepository>,
    store_service: Arc<StoreService>,
    llm_provider: Arc<dyn LlmProvider>,
}


impl ChatService {
    pub fn new(
        chat_repo: Arc<ChatRepository>,
        store_service: Arc<StoreService>,
        llm_provider: Arc<dyn LlmProvider>,
    ) -> Self {
        Self {
            chat_repo,
            store_service,
            llm_provider,
        }
    }

    pub async fn ask_question(
        &self,
        user_id: Uuid,
        store_id: Uuid,
        room_id: Option<Uuid>,
        question: String,
    ) -> Result<ChatAskResult, ChatServiceError> {
        let _store = self.store_service.get_store(user_id, store_id).await?;

        let room = if let Some(r_id) = room_id {
            let existing_room = self.chat_repo.find_room_by_id(r_id).await
                .map_err(|e| ChatServiceError::InternalError(e.to_string()))?
                .ok_or(ChatServiceError::InternalError("Room not found".to_string()))?;

            if existing_room.user_id != user_id.to_string() {
                return Err(ChatServiceError::StoreNotFound);
            }
            existing_room
        } else {
            let new_room_id = Uuid::new_v4();
            let title = if question.chars().count() > 40 {
                format!("{}...", &question[..40])
            } else {
                question.clone()
            };

            self.chat_repo.create_room(new_room_id, user_id, Some(store_id), Some(&title)).await
                .map_err(|e| ChatServiceError::InternalError(e.to_string()))?
        };

        let room_uuid = Uuid::parse_str(&room.id).unwrap();

        let user_msg_id = Uuid::new_v4();
        let user_message = self.chat_repo.create_message(
            user_msg_id,
            room_uuid,
            "user",
            &question,
            None, None, None, false, None,
        ).await.map_err(|e| ChatServiceError::InternalError(e.to_string()))?;

        let mut llm_messages = vec![
            LlmMessage {
                role: "system".to_string(),
                content: "You are a helpful e-commerce product support assistant. Answer concisely based on the provided context. If you don't know, say so.".to_string(),
            }
        ];

        let history = self.chat_repo.get_recent_messages(room_uuid, 10).await
            .map_err(|e| ChatServiceError::InternalError(e.to_string()))?;

        for msg in history {
            llm_messages.push(LlmMessage {
                role: msg.role,
                content: msg.content,
            });
        }

        let start_time = std::time::Instant::now();
        let llm_response = self.llm_provider.chat(llm_messages).await?;
        let latency_ms = start_time.elapsed().as_millis() as i32;

        let assistant_msg_id = Uuid::new_v4();
        let assistant_message = self.chat_repo.create_message(
            assistant_msg_id,
            room_uuid,
            "assistant",
            &llm_response.content,
            None, // sources_json (появится после интеграции Qdrant)
            Some("qwen2.5:1.5b"),
            Some(latency_ms),
            false,
            None,
        ).await.map_err(|e| ChatServiceError::InternalError(e.to_string()))?;

        Ok(ChatAskResult {
            room,
            user_message,
            assistant_message,
        })
    }
}