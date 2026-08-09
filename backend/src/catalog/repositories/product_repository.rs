use anyhow::Result;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::catalog::entities::product::Product;

pub struct ProductRepository {
    pool: SqlitePool,
}

impl ProductRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        id: Uuid,
        store_id: Uuid,
        name: &str,
        category: Option<&str>,
        description: Option<&str>,
        specs: Option<&str>,
        manual: Option<&str>,
        faq: Option<&str>,
        external_id: Option<&str>,
        source: &str,
    ) -> Result<Product> {
        let id_str = id.to_string();
        let store_id_str = store_id.to_string();

        sqlx::query(
            r#"
            INSERT INTO products (id, store_id, name, category, description, specs, manual, faq, external_id, source, status)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'active')
            "#,
        )
            .bind(&id_str)
            .bind(&store_id_str)
            .bind(name)
            .bind(category)
            .bind(description)
            .bind(specs)
            .bind(manual)
            .bind(faq)
            .bind(external_id)
            .bind(source)
            .execute(&self.pool)
            .await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to fetch created product"))
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>> {
        let id_str = id.to_string();

        let product = sqlx::query_as::<_, Product>(
            r#"
            SELECT id, store_id, name, category, description, specs, manual, faq, status, external_id, source, created_at, updated_at
            FROM products
            WHERE id = ?
            "#,
        )
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await?;

        Ok(product)
    }

    pub async fn find_by_store_id(&self, store_id: Uuid) -> Result<Vec<Product>> {
        let store_id_str = store_id.to_string();

        let products = sqlx::query_as::<_, Product>(
            r#"
            SELECT id, store_id, name, category, description, specs, manual, faq, status, external_id, source, created_at, updated_at
            FROM products
            WHERE store_id = ?
            ORDER BY created_at DESC
            "#,
        )
            .bind(&store_id_str)
            .fetch_all(&self.pool)
            .await?;

        Ok(products)
    }

    pub async fn update(
        &self,
        id: Uuid,
        name: &str,
        category: Option<&str>,
        description: Option<&str>,
        specs: Option<&str>,
        manual: Option<&str>,
        faq: Option<&str>,
        status: &str,
        external_id: Option<&str>,
        source: &str,
    ) -> Result<Option<Product>> {
        let id_str = id.to_string();

        let result = sqlx::query(
            r#"
            UPDATE products
            SET name = ?,
                category = ?,
                description = ?,
                specs = ?,
                manual = ?,
                faq = ?,
                status = ?,
                external_id = ?,
                source = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
            .bind(name)
            .bind(category)
            .bind(description)
            .bind(specs)
            .bind(manual)
            .bind(faq)
            .bind(status)
            .bind(external_id)
            .bind(source)
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

        let result = sqlx::query("DELETE FROM products WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn count_by_store_id(&self, store_id: Uuid) -> Result<i64> {
        let store_id_str = store_id.to_string();

        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM products WHERE store_id = ?
            "#,
        )
            .bind(&store_id_str)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }
}