use serde::Serialize;
use crate::system::domain::service_status::ServiceStatus;

#[derive(Serialize)]
pub struct ServicesHealth {
    pub version: &'static str,
    pub uptime_seconds: i64,
    pub database: ServiceStatus,
    pub qdrant: ServiceStatus,
    pub llm: ServiceStatus,
    pub embeddings: ServiceStatus,
}