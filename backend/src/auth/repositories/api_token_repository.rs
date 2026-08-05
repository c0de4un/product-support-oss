use std::sync::Arc;
use sqlx::SqlitePool;
use sha2::{Sha256, Digest};
use moka::sync::Cache;
use crate::auth::entities::api_token::ApiToken;

pub struct ApiTokenRepository {
    pool: SqlitePool,
    tokens_cache: Cache<String, Arc<ApiToken>>,
}

impl ApiTokenRepository {
    pub fn new(pool: SqlitePool, tokens_cache: Cache<String, Arc<ApiToken>>) -> Self {
        Self {
            pool,
            tokens_cache,
        }
    }

    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub async fn create(
        &self,
        id: &str,
        user_id: &str,
        token_hash: &str,
        name: &str,
    ) -> Result<ApiToken, sqlx::Error> {
        let token = sqlx::query_as::<_, ApiToken>(
            "INSERT INTO api_tokens (id, user_id, token_hash, name) VALUES (?, ?, ?, ?) RETURNING *",
        )
            .bind(id)
            .bind(user_id)
            .bind(token_hash)
            .bind(name)
            .fetch_one(&self.pool)
            .await?;

        Ok(token)
    }

    pub async fn find_by_hash(&self, token_hash: &str) -> Result<Option<Arc<ApiToken>>, sqlx::Error> {
        if let Some(token) = self.tokens_cache.get(&token_hash.to_string()) {
            return Ok(Some(token));
        }

        let token = sqlx::query_as::<_, ApiToken>("SELECT * FROM api_tokens WHERE token_hash = ?")
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(t) = token {
            let arc_token = Arc::new(t);
            self.tokens_cache.insert(token_hash.to_string(), arc_token.clone());
            return Ok(Some(arc_token));
        }

        Ok(None)
    }

    pub async fn touch_last_used(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE api_tokens SET last_used_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        let token = sqlx::query_as::<_, ApiToken>("SELECT * FROM api_tokens WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(t) = token {
            self.tokens_cache.invalidate(&t.token_hash);
        }

        sqlx::query("DELETE FROM api_tokens WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}