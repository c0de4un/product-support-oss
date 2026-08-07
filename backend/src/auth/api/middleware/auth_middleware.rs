use std::sync::Arc;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::auth::services::auth_service::verify_jwt;
use crate::config::state::AppState;

pub struct AuthUser {
    pub id: Uuid,
}

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Missing or invalid Authorization header"})),
                )
            })?;

        let claims = match verify_jwt(token, &state.config) {
            Ok(c) => c,
            Err(_) => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Invalid or expired token"})),
                ));
            }
        };

        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR, // Или UNAUTHORIZED, если считаем, что токен битый
                Json(json!({"error": "Invalid user ID format in token"})),
            )
        })?;

        Ok(AuthUser { id: user_id })
    }
}