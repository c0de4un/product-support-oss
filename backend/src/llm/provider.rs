use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Provider error: {0}")]
    Provider(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub prompt_tokens: Option<i32>,
    pub completion_tokens: Option<i32>,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat(&self, messages: Vec<LlmMessage>) -> Result<LlmResponse, LlmError>;
}