use sqlx::SqlitePool;
use chrono::{DateTime, Utc};
use moka::sync::Cache;
use crate::config::config::Config;
use crate::user::repositories::user_repository::UserRepository;
use crate::auth::repositories::api_token_repository::ApiTokenRepository;

pub struct AppState {
    pub db: SqlitePool,
    pub config: Config,
    pub started_at: DateTime<Utc>,
    pub user_repository: UserRepository,
    pub api_token_repository: ApiTokenRepository,
}

impl AppState {
    pub fn new(db: SqlitePool, config: Config) -> Self {
        let tokens_cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(std::time::Duration::from_secs(600))
            .build();

        let user_repository = UserRepository::new(db.clone());

        let api_token_repository = ApiTokenRepository::new(db.clone(), tokens_cache);

        Self {
            db,
            config,
            started_at: Utc::now(),
            user_repository,
            api_token_repository,
        }
    }
}