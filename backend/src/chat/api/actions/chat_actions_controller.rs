use std::sync::Arc;
use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    Json,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tracing::error;
use validator::Validate;

use crate::auth::api::middleware::auth_middleware::AuthUser;
use crate::chat::api::requests::chat_request::ChatRequest;
use crate::chat::api::responses::chat_response::ChatResponse;
use crate::chat::services::chat_service::{ChatAskResult, ChatServiceError};
use crate::config::state::AppState;

type ApiResult<T> = Result<T, (StatusCode, Json<Value>)>;

fn parse_json<T: DeserializeOwned>(payload: Result<Json<T>, JsonRejection>) -> ApiResult<T> {
    payload.map(|Json(p)| p).map_err(|rejection| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "errors": { "detail": rejection.body_text() } })),
        )
    })
}

fn handle_validation_errors(errors: validator::ValidationErrors) -> (StatusCode, Json<Value>) {
    let mut error_map = serde_json::Map::new();
    for (field, field_errors) in errors.field_errors() {
        let messages: Vec<String> = field_errors
            .iter()
            .map(|e| e.message.as_ref().map(|m| m.to_string()).unwrap_or_else(|| e.code.to_string()))
            .collect();
        error_map.insert(field.to_string(), Value::String(messages.join(", ")));
    }
    (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({ "errors": error_map })))
}

fn map_chat_service_error(err: ChatServiceError) -> (StatusCode, Json<Value>) {
    match err {
        ChatServiceError::StoreNotFound => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Store not found or access denied" })),
        ),
        ChatServiceError::LlmError(msg) => {
            error!("LLM provider error: {}", msg);
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": "AI service temporarily unavailable" })),
            )
        }
        ChatServiceError::InternalError(msg) => {
            error!("Chat service internal error: {}", msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            )
        }
    }
}

pub async fn ask_question(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    payload_result: Result<Json<ChatRequest>, JsonRejection>,
) -> ApiResult<(StatusCode, Json<ChatResponse>)> {
    let payload = parse_json(payload_result)?;
    payload.validate().map_err(handle_validation_errors)?;

    let result: ChatAskResult = state
        .chat_service
        .ask_question(
            user.id,
            payload.store_id,
            payload.room_id,
            payload.question,
        )
        .await
        .map_err(map_chat_service_error)?;

    let response = ChatResponse::new(result.room, result.user_message, result.assistant_message);

    Ok((StatusCode::OK, Json(response)))
}