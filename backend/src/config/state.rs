use sqlx::SqlitePool;
use chrono::{DateTime, Utc};
use crate::config::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Config,
    pub started_at: DateTime<Utc>,
}

impl AppState {
    pub fn new(db: SqlitePool, config: Config) -> Self {
        Self {
            db,
            config,
            started_at: Utc::now(),
        }
    }
}