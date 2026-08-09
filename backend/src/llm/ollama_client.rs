use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::provider::{LlmError, LlmMessage, LlmProvider, LlmResponse};

pub struct OllamaClient {
    client: Client,
    base_url: String,
    model: String,
}

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessage,
    prompt_eval_count: Option<i32>,
    eval_count: Option<i32>,
}

impl OllamaClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            model,
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaClient {
    async fn chat(&self, messages: Vec<LlmMessage>) -> Result<LlmResponse, LlmError> {
        let req_messages = messages
            .into_iter()
            .map(|m| OllamaMessage {
                role: m.role,
                content: m.content,
            })
            .collect();

        let req_body = OllamaChatRequest {
            model: self.model.clone(),
            messages: req_messages,
            stream: false,
        };

        let url = format!("{}/api/chat", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::Provider(format!("Status {}: {}", status, text)));
        }

        let ollama_resp: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| LlmError::Provider(e.to_string()))?;

        Ok(LlmResponse {
            content: ollama_resp.message.content,
            prompt_tokens: ollama_resp.prompt_eval_count,
            completion_tokens: ollama_resp.eval_count,
        })
    }
}