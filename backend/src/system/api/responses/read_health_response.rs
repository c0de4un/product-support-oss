use serde::Serialize;
use crate::system::domain::service_health::ServicesHealth;

#[derive(Serialize)]
pub struct HealthResponse {
    pub health: ServicesHealth,
}