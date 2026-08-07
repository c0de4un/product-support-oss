use anyhow::Result;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::store::entities::store::Store;

pub struct StoreRepository {
    pool: SqlitePool,
}

impl StoreRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: &str,
        domain: Option<&str>,
        description: Option<&str>,
        api_token: Option<&str>,
    ) -> Result<Store> {
        let id_str = id.to_string();
        let user_id_str = user_id.to_string();

        sqlx::query(
            r#"
            INSERT INTO stores (id, user_id, name, domain, description, api_token, status)
            VALUES (?, ?, ?, ?, ?, ?, 'active')
            "#,
        )
            .bind(&id_str)
            .bind(&user_id_str)
            .bind(name)
            .bind(domain)
            .bind(description)
            .bind(api_token)
            .execute(&self.pool)
            .await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to fetch created store"))
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Store>> {
        let id_str = id.to_string();

        let store = sqlx::query_as::<_, Store>(
            r#"
            SELECT id, user_id, name, domain, description, api_token, status, created_at, updated_at
            FROM stores
            WHERE id = ?
            "#,
        )
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await?;

        Ok(store)
    }

    pub async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Store>> {
        let user_id_str = user_id.to_string();

        let stores = sqlx::query_as::<_, Store>(
            r#"
            SELECT id, user_id, name, domain, description, api_token, status, created_at, updated_at
            FROM stores
            WHERE user_id = ?
            ORDER BY created_at DESC
            "#,
        )
            .bind(&user_id_str)
            .fetch_all(&self.pool)
            .await?;

        Ok(stores)
    }

    pub async fn find_by_api_token(&self, api_token: &str) -> Result<Option<Store>> {
        let store = sqlx::query_as::<_, Store>(
            r#"
            SELECT id, user_id, name, domain, description, api_token, status, created_at, updated_at
            FROM stores
            WHERE api_token = ? AND status = 'active'
            "#,
        )
            .bind(api_token)
            .fetch_optional(&self.pool)
            .await?;

        Ok(store)
    }

    pub async fn find_by_domain(&self, domain: &str) -> Result<Option<Store>> {
        let store = sqlx::query_as::<_, Store>(
            r#"
        SELECT id, user_id, name, domain, description, api_token, status, created_at, updated_at
        FROM stores
        WHERE domain = ?
        "#,
        )
            .bind(domain)
            .fetch_optional(&self.pool)
            .await?;

        Ok(store)
    }

    pub async fn update(
        &self,
        id: Uuid,
        name: &str,
        domain: Option<&str>,
        description: Option<&str>,
        api_token: Option<&str>, // <--- Добавили
        status: &str,
    ) -> Result<Option<Store>> {
        let id_str = id.to_string();

        let result = sqlx::query(
            r#"
            UPDATE stores
            SET name = ?, domain = ?, description = ?, api_token = ?, status = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
            .bind(name)
            .bind(domain)
            .bind(description)
            .bind(api_token)
            .bind(status)
            .bind(&id_str)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        self.find_by_id(id).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let id_str = id.to_string();

        let result = sqlx::query("DELETE FROM stores WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}