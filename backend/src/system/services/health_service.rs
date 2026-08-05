use std::sync::Arc;
use crate::system::domain::service_health::ServicesHealth;
use crate::system::domain::service_status::ServiceStatus;
use crate::config::state::AppState;

pub struct HealthService;

impl HealthService {
    pub fn read(state: Arc<AppState>) -> ServicesHealth {
        let uptime = chrono::Utc::now()
            .signed_duration_since(state.started_at)
            .num_seconds();

        ServicesHealth {
            version: env!("CARGO_PKG_VERSION"),
            uptime_seconds: uptime,
            database: ServiceStatus::pending(),
            qdrant: ServiceStatus::pending(),
            llm: ServiceStatus::pending(),
            embeddings: ServiceStatus::pending(),
        }
    }
}