use std::sync::Arc;
use chrono::{DateTime, Utc};
use moka::sync::Cache;
use sqlx::SqlitePool;

use crate::auth::repositories::api_token_repository::ApiTokenRepository;
use crate::config::config::Config;
use crate::store::repositories::store_repository::StoreRepository;
use crate::user::repositories::user_repository::UserRepository;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Arc<Config>,
    pub started_at: DateTime<Utc>,
    pub user_repository: Arc<UserRepository>,
    pub api_token_repository: Arc<ApiTokenRepository>,
    pub store_repository: Arc<StoreRepository>,
}

impl AppState {
    pub fn new(db: SqlitePool, config: Config) -> Self {
        let tokens_cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(std::time::Duration::from_secs(600))
            .build();

        let user_repository = Arc::new(UserRepository::new(db.clone()));
        let api_token_repository = Arc::new(ApiTokenRepository::new(db.clone(), tokens_cache));
        let store_repository = Arc::new(StoreRepository::new(db.clone()));

        Self {
            db,
            config: Arc::new(config),
            started_at: Utc::now(),
            user_repository,
            api_token_repository,
            store_repository,
        }
    }
}