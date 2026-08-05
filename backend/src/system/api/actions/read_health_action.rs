use std::sync::Arc;
use axum::{
    extract::{State},
    Json,
};
use crate::system::api::responses::read_health_response::HealthResponse;
use crate::system::services::health_service::HealthService;
use crate::config::state::AppState;

pub async fn read_health_action(State(state): State<Arc<AppState>>,) -> Json<HealthResponse> {
    let response = HealthResponse {
        health: HealthService::read(state),
    };

    Json(response)
}
