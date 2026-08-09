use std::sync::Arc;
use chrono::{DateTime, Utc};
use moka::sync::Cache;
use sqlx::SqlitePool;

use crate::auth::repositories::api_token_repository::ApiTokenRepository;
use crate::config::config::Config;
use crate::store::repositories::store_repository::StoreRepository;
use crate::store::services::store_service::StoreService;
use crate::user::repositories::user_repository::UserRepository;
use crate::chat::repositories::chat_repository::ChatRepository;
use crate::chat::services::chat_service::ChatService;
use crate::llm::ollama_client::OllamaClient;
use crate::llm::provider::LlmProvider;
use crate::catalog::repositories::product_repository::ProductRepository;
use crate::catalog::services::product_service::ProductService;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Arc<Config>,
    pub started_at: DateTime<Utc>,
    pub user_repository: Arc<UserRepository>,
    pub api_token_repository: Arc<ApiTokenRepository>,
    pub store_repository: Arc<StoreRepository>,
    pub store_service: Arc<StoreService>,
    pub chat_repository: Arc<ChatRepository>,
    pub chat_service: Arc<ChatService>,
    pub product_repository: Arc<ProductRepository>,
    pub product_service: Arc<ProductService>,
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

        let store_service = Arc::new(StoreService::new(store_repository.clone()));

        let chat_repository = Arc::new(ChatRepository::new(db.clone()));

        let llm_client: Arc<dyn LlmProvider> = Arc::new(OllamaClient::new(
            config.llm_base_url.clone(),
            config.llm_model.clone(),
        ));

        let chat_service = Arc::new(ChatService::new(
            chat_repository.clone(),
            store_service.clone(),
            llm_client,
        ));

        let product_repository = Arc::new(ProductRepository::new(db.clone()));

        let product_service = Arc::new(ProductService::new(
            product_repository.clone(),
            store_repository.clone(),
        ));

        Self {
            db,
            config: Arc::new(config),
            started_at: Utc::now(),
            user_repository,
            api_token_repository,
            store_repository,
            store_service,
            chat_repository,
            chat_service,
            product_repository,
            product_service,
        }
    }
}