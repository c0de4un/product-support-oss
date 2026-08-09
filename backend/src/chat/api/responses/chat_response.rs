use serde::Serialize;

use crate::chat::entities::chat_message::ChatMessage;
use crate::chat::entities::chat_room::ChatRoom;

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub room: ChatRoom,
    pub user_message: ChatMessage,
    pub assistant_message: ChatMessage,
}

impl ChatResponse {
    pub fn new(room: ChatRoom, user_message: ChatMessage, assistant_message: ChatMessage) -> Self {
        Self {
            room,
            user_message,
            assistant_message,
        }
    }
}