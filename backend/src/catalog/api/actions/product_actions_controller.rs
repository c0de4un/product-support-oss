use std::sync::Arc;
use axum::{
    extract::{rejection::JsonRejection, Path, State},
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
use crate::catalog::api::requests::{
    create_product_request::CreateProductRequest,
    update_product_request::UpdateProductRequest,
};
use crate::catalog::api::responses::product_response::ProductResponse;
use crate::catalog::services::product_service::ProductServiceError;

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

fn map_product_service_error(err: ProductServiceError) -> (StatusCode, Json<Value>) {
    match err {
        ProductServiceError::NotFound => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Product not found" })),
        ),
        ProductServiceError::StoreNotFound => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Store not found" })),
        ),
        ProductServiceError::Forbidden => (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Access denied" })),
        ),
        ProductServiceError::InvalidSpecsFormat => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "errors": {
                    "specs": "Specs must be a JSON object"
                }
            })),
        ),
        ProductServiceError::InvalidFaqFormat => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "errors": {
                    "faq": "FAQ must be a JSON array"
                }
            })),
        ),
        ProductServiceError::InternalError(message) => {
            error!("Product service error: {}", message);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            )
        }
    }
}

pub async fn create_product(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(store_id): Path<Uuid>,
    payload_result: Result<Json<CreateProductRequest>, JsonRejection>,
) -> ApiResult<(StatusCode, Json<ProductResponse>)> {
    let payload = parse_json(payload_result)?;
    payload.validate().map_err(handle_validation_errors)?;

    let product = state
        .product_service
        .create_product(
            user.id,
            store_id,
            payload.name,
            payload.category,
            payload.description,
            payload.specs,
            payload.manual,
            payload.faq,
            payload.external_id,
            payload.source,
        )
        .await
        .map_err(map_product_service_error)?;

    Ok((
        StatusCode::CREATED,
        Json(ProductResponse::from(product)),
    ))
}

pub async fn list_products(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(store_id): Path<Uuid>,
) -> ApiResult<Json<Vec<ProductResponse>>> {
    let products = state
        .product_service
        .list_products(user.id, store_id)
        .await
        .map_err(map_product_service_error)?;

    let response: Vec<ProductResponse> = products
        .into_iter()
        .map(ProductResponse::from)
        .collect();

    Ok(Json(response))
}

pub async fn get_product(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
) -> ApiResult<Json<ProductResponse>> {
    let product = state
        .product_service
        .get_product(user.id, product_id)
        .await
        .map_err(map_product_service_error)?;

    Ok(Json(ProductResponse::from(product)))
}

pub async fn update_product(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
    payload_result: Result<Json<UpdateProductRequest>, JsonRejection>,
) -> ApiResult<Json<ProductResponse>> {
    let payload = parse_json(payload_result)?;
    payload.validate().map_err(handle_validation_errors)?;

    let product = state
        .product_service
        .update_product(
            user.id,
            product_id,
            payload.name,
            payload.category,
            payload.description,
            payload.specs,
            payload.manual,
            payload.faq,
            payload.status,
            payload.external_id,
            payload.source,
        )
        .await
        .map_err(map_product_service_error)?;

    Ok(Json(ProductResponse::from(product)))
}

pub async fn delete_product(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state
        .product_service
        .delete_product(user.id, product_id)
        .await
        .map_err(map_product_service_error)?;

    Ok(StatusCode::NO_CONTENT)
}