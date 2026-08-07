use std::sync::Arc;
use axum::{
    extract::{rejection::JsonRejection, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tracing::error;
use uuid::Uuid;
use validator::Validate;

use crate::auth::api::middleware::auth_middleware::AuthUser;
use crate::config::state::AppState;
use crate::store::api::requests::{
    create_store_request::CreateStoreRequest,
    delete_store_request::DeleteStoreRequest,
    update_store_request::UpdateStoreRequest,
};
use crate::store::api::responses::store_response::StoreResponse;
use crate::store::services::store_service::StoreServiceError;

type ApiResult<T> = Result<T, (StatusCode, Json<Value>)>;

fn json_error(status: StatusCode, msg: &str) -> (StatusCode, Json<Value>) {
    (status, Json(json!({ "error": msg })))
}

fn parse_json<T: DeserializeOwned>(
    payload: Result<Json<T>, JsonRejection>,
) -> ApiResult<T> {
    payload.map(|Json(p)| p).map_err(|rejection| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "errors": {
                    "detail": rejection.body_text()
                }
            })),
        )
    })
}

fn handle_validation_errors(
    errors: validator::ValidationErrors,
) -> (StatusCode, Json<Value>) {
    let mut error_map = serde_json::Map::new();

    for (field, field_errors) in errors.field_errors() {
        let messages: Vec<String> = field_errors
            .iter()
            .map(|e| {
                e.message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| e.code.to_string())
            })
            .collect();

        error_map.insert(
            field.to_string(),
            Value::String(messages.join(", ")),
        );
    }

    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(json!({ "errors": error_map })),
    )
}

fn map_store_service_error(err: StoreServiceError) -> (StatusCode, Json<Value>) {
    match err {
        StoreServiceError::NotFound => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Store not found" })),
        ),

        StoreServiceError::DomainAlreadyExists => (
            StatusCode::CONFLICT,
            Json(json!({
                "errors": {
                    "domain": "Domain already exists"
                }
            })),
        ),

        StoreServiceError::InternalError(message) => {
            error!("Store service error: {}", message);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            )
        }
    }
}

pub async fn create_store(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    payload_result: Result<Json<CreateStoreRequest>, JsonRejection>,
) -> ApiResult<(StatusCode, Json<StoreResponse>)> {
    let payload = parse_json(payload_result)?;
    payload.validate().map_err(handle_validation_errors)?;

    let store = state
        .store_service
        .create_store(
            user.id,
            payload.name,
            payload.domain,
            payload.description,
        )
        .await
        .map_err(map_store_service_error)?;

    Ok((
        StatusCode::CREATED,
        Json(StoreResponse::with_api_token(store)),
    ))
}

pub async fn list_stores(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<StoreResponse>>> {
    let stores = state
        .store_service
        .list_stores(user.id)
        .await
        .map_err(map_store_service_error)?;

    let response: Vec<StoreResponse> = stores
        .into_iter()
        .map(StoreResponse::public)
        .collect();

    Ok(Json(response))
}

pub async fn get_store(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(store_id): Path<Uuid>,
) -> ApiResult<Json<StoreResponse>> {
    let store = state
        .store_service
        .get_store(user.id, store_id)
        .await
        .map_err(map_store_service_error)?;

    Ok(Json(StoreResponse::public(store)))
}

pub async fn update_store(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(store_id): Path<Uuid>,
    payload_result: Result<Json<UpdateStoreRequest>, JsonRejection>,
) -> ApiResult<Json<StoreResponse>> {
    let payload = parse_json(payload_result)?;
    payload.validate().map_err(handle_validation_errors)?;

    let store = state
        .store_service
        .update_store(
            user.id,
            store_id,
            payload.name,
            payload.domain,
            payload.description,
        )
        .await
        .map_err(map_store_service_error)?;

    Ok(Json(StoreResponse::public(store)))
}

pub async fn delete_store(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(store_id): Path<Uuid>,
    Query(query): Query<DeleteStoreRequest>,
) -> ApiResult<StatusCode> {
    if query.confirm == Some(false) {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "Deletion was not confirmed",
        ));
    }

    state
        .store_service
        .delete_store(user.id, store_id)
        .await
        .map_err(map_store_service_error)?;

    Ok(StatusCode::NO_CONTENT)
}