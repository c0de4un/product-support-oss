use std::sync::Arc;
use serde_json::Value;
use uuid::Uuid;

use crate::catalog::entities::product::Product;
use crate::catalog::repositories::product_repository::ProductRepository;
use crate::store::repositories::store_repository::StoreRepository;

#[derive(Debug)]
pub enum ProductServiceError {
    NotFound,
    StoreNotFound,
    Forbidden,
    InvalidSpecsFormat,
    InvalidFaqFormat,
    InternalError(String),
}

impl std::fmt::Display for ProductServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductServiceError::NotFound => write!(f, "Product not found"),
            ProductServiceError::StoreNotFound => write!(f, "Store not found"),
            ProductServiceError::Forbidden => write!(f, "Access denied"),
            ProductServiceError::InvalidSpecsFormat => write!(f, "Specs must be a JSON object"),
            ProductServiceError::InvalidFaqFormat => write!(f, "FAQ must be a JSON array"),
            ProductServiceError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ProductServiceError {}

pub struct ProductService {
    product_repo: Arc<ProductRepository>,
    store_repo: Arc<StoreRepository>,
}

impl ProductService {
    pub fn new(
        product_repo: Arc<ProductRepository>,
        store_repo: Arc<StoreRepository>,
    ) -> Self {
        Self {
            product_repo,
            store_repo,
        }
    }

    async fn check_store_access(
        &self,
        user_id: Uuid,
        store_id: Uuid,
    ) -> Result<(), ProductServiceError> {
        let store = self.store_repo.find_by_id(store_id)
            .await
            .map_err(|e| ProductServiceError::InternalError(e.to_string()))?
            .ok_or(ProductServiceError::StoreNotFound)?;

        if store.user_id != user_id.to_string() {
            return Err(ProductServiceError::Forbidden);
        }

        Ok(())
    }

    async fn get_product_with_access(
        &self,
        user_id: Uuid,
        product_id: Uuid,
    ) -> Result<Product, ProductServiceError> {
        let product = self.product_repo.find_by_id(product_id)
            .await
            .map_err(|e| ProductServiceError::InternalError(e.to_string()))?
            .ok_or(ProductServiceError::NotFound)?;

        let store_id = Uuid::parse_str(&product.store_id)
            .map_err(|_| ProductServiceError::InternalError("Invalid store_id format".to_string()))?;

        self.check_store_access(user_id, store_id).await?;

        Ok(product)
    }

    pub async fn create_product(
        &self,
        user_id: Uuid,
        store_id: Uuid,
        name: String,
        category: Option<String>,
        description: Option<String>,
        specs: Option<Value>,
        manual: Option<String>,
        faq: Option<Value>,
        external_id: Option<String>,
        source: Option<String>,
    ) -> Result<Product, ProductServiceError> {
        self.check_store_access(user_id, store_id).await?;

        if let Some(ref s) = specs {
            if !s.is_object() {
                return Err(ProductServiceError::InvalidSpecsFormat);
            }
        }

        if let Some(ref f) = faq {
            if !f.is_array() {
                return Err(ProductServiceError::InvalidFaqFormat);
            }
        }

        let specs_str = specs.map(|v| v.to_string());
        let faq_str = faq.map(|v| v.to_string());

        let product_id = Uuid::new_v4();
        let source_val = source.unwrap_or_else(|| "manual".to_string());

        self.product_repo
            .create(
                product_id,
                store_id,
                &name,
                category.as_deref(),
                description.as_deref(),
                specs_str.as_deref(),
                manual.as_deref(),
                faq_str.as_deref(),
                external_id.as_deref(),
                &source_val,
            )
            .await
            .map_err(|e| ProductServiceError::InternalError(e.to_string()))
    }

    pub async fn get_product(
        &self,
        user_id: Uuid,
        product_id: Uuid,
    ) -> Result<Product, ProductServiceError> {
        self.get_product_with_access(user_id, product_id).await
    }

    pub async fn list_products(
        &self,
        user_id: Uuid,
        store_id: Uuid,
    ) -> Result<Vec<Product>, ProductServiceError> {
        self.check_store_access(user_id, store_id).await?;

        self.product_repo
            .find_by_store_id(store_id)
            .await
            .map_err(|e| ProductServiceError::InternalError(e.to_string()))
    }

    pub async fn update_product(
        &self,
        user_id: Uuid,
        product_id: Uuid,
        name: String,
        category: Option<String>,
        description: Option<String>,
        specs: Option<Value>,
        manual: Option<String>,
        faq: Option<Value>,
        status: Option<String>,
        external_id: Option<String>,
        source: Option<String>,
    ) -> Result<Product, ProductServiceError> {
        let existing_product = self.get_product_with_access(user_id, product_id).await?;

        // Валидация JSON
        if let Some(ref s) = specs {
            if !s.is_object() {
                return Err(ProductServiceError::InvalidSpecsFormat);
            }
        }
        if let Some(ref f) = faq {
            if !f.is_array() {
                return Err(ProductServiceError::InvalidFaqFormat);
            }
        }

        let specs_str = specs.map(|v| v.to_string());
        let faq_str = faq.map(|v| v.to_string());

        self.product_repo
            .update(
                product_id,
                &name,
                category.as_deref(),
                description.as_deref(),
                specs_str.as_deref(),
                manual.as_deref(),
                faq_str.as_deref(),
                &status.unwrap_or(existing_product.status),
                external_id.as_deref(),
                &source.unwrap_or(existing_product.source),
            )
            .await
            .map_err(|e| ProductServiceError::InternalError(e.to_string()))?
            .ok_or(ProductServiceError::NotFound)
    }

    pub async fn delete_product(
        &self,
        user_id: Uuid,
        product_id: Uuid,
    ) -> Result<(), ProductServiceError> {
        self.get_product_with_access(user_id, product_id).await?;

        let deleted = self.product_repo.delete(product_id)
            .await
            .map_err(|e| ProductServiceError::InternalError(e.to_string()))?;

        if !deleted {
            return Err(ProductServiceError::NotFound);
        }

        Ok(())
    }
}