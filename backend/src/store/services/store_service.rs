use std::sync::Arc;
use uuid::Uuid;

use crate::store::entities::store::Store;
use crate::store::repositories::store_repository::StoreRepository;

#[derive(Debug)]
pub enum StoreServiceError {
    NotFound,
    DomainAlreadyExists,
    InternalError(String),
}

impl std::fmt::Display for StoreServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreServiceError::NotFound => write!(f, "Store not found"),
            StoreServiceError::DomainAlreadyExists => write!(f, "Domain already exists"),
            StoreServiceError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for StoreServiceError {}

pub struct StoreService {
    repo: Arc<StoreRepository>,
}

impl StoreService {
    pub fn new(repo: Arc<StoreRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_store(
        &self,
        user_id: Uuid,
        name: String,
        domain: Option<String>,
        description: Option<String>,
    ) -> Result<Store, StoreServiceError> {
        let store_id = Uuid::new_v4();

        let normalized_domain = domain
            .map(|d| d.trim().to_lowercase())
            .filter(|d| !d.is_empty());

        if let Some(ref dom) = normalized_domain {
            if self.repo.find_by_domain(dom).await
                .map_err(|e| StoreServiceError::InternalError(e.to_string()))?
                .is_some()
            {
                return Err(StoreServiceError::DomainAlreadyExists);
            }
        }

        let api_token = Some(format!("{}", Uuid::new_v4().to_string().replace("-", "")));

        self.repo
            .create(
                store_id,
                user_id,
                &name,
                normalized_domain.as_deref(),
                description.as_deref(),
                api_token.as_deref(),
            )
            .await
            .map_err(|e| {
                if let Some(sqlx::Error::Database(db_err)) = e.downcast_ref::<sqlx::Error>() {
                    if db_err.message().contains("FOREIGN KEY constraint failed") {
                        return StoreServiceError::NotFound;
                    }
                }

                StoreServiceError::InternalError(e.to_string())
            })
    }

    pub async fn get_store(&self, user_id: Uuid, store_id: Uuid) -> Result<Store, StoreServiceError> {
        let store = self.repo.find_by_id(store_id).await
            .map_err(|e| StoreServiceError::InternalError(e.to_string()))?
            .ok_or(StoreServiceError::NotFound)?;

        if store.user_id != user_id.to_string() {
            return Err(StoreServiceError::NotFound);
        }

        Ok(store)
    }

    pub async fn list_stores(&self, user_id: Uuid) -> Result<Vec<Store>, StoreServiceError> {
        self.repo.find_by_user_id(user_id).await
            .map_err(|e| StoreServiceError::InternalError(e.to_string()))
    }

    pub async fn update_store(
        &self,
        user_id: Uuid,
        store_id: Uuid,
        name: String,
        domain: Option<String>,
        description: Option<String>,
    ) -> Result<Store, StoreServiceError> {
        let existing_store = self.get_store(user_id, store_id).await?;

        let normalized_domain = domain
            .map(|d| d.trim().to_lowercase())
            .filter(|d| !d.is_empty());

        if let Some(ref dom) = normalized_domain {
            if Some(dom.as_str()) != existing_store.domain.as_deref() {
                if self.repo.find_by_domain(dom).await
                    .map_err(|e| StoreServiceError::InternalError(e.to_string()))?
                    .is_some()
                {
                    return Err(StoreServiceError::DomainAlreadyExists);
                }
            }
        }

        self.repo
            .update(
                store_id,
                &name,
                normalized_domain.as_deref(),
                description.as_deref(),
                existing_store.api_token.as_deref(), // Сохраняем текущий токен
                &existing_store.status,
            )
            .await
            .map_err(|e| StoreServiceError::InternalError(e.to_string()))?
            .ok_or(StoreServiceError::NotFound)
    }

    pub async fn delete_store(&self, user_id: Uuid, store_id: Uuid) -> Result<(), StoreServiceError> {
        self.get_store(user_id, store_id).await?;

        let deleted = self.repo.delete(store_id).await
            .map_err(|e| StoreServiceError::InternalError(e.to_string()))?;

        if !deleted {
            return Err(StoreServiceError::NotFound);
        }

        Ok(())
    }

    pub async fn rotate_api_token(&self, user_id: Uuid, store_id: Uuid) -> Result<Store, StoreServiceError> {
        let existing_store = self.get_store(user_id, store_id).await?;
        let new_token = format!("{}", Uuid::new_v4().to_string().replace("-", ""));

        self.repo
            .update(
                store_id,
                &existing_store.name,
                existing_store.domain.as_deref(),
                existing_store.description.as_deref(),
                Some(&new_token),
                &existing_store.status,
            )
            .await
            .map_err(|e| StoreServiceError::InternalError(e.to_string()))?
            .ok_or(StoreServiceError::NotFound)
    }
}