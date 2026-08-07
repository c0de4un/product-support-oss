use anyhow::Result;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::chat::entities::chat_message::ChatMessage;
use crate::chat::entities::chat_room::ChatRoom;

pub struct ChatRepository {
    pool: SqlitePool,
}

impl ChatRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_room(
        &self,
        id: Uuid,
        user_id: Uuid,
        store_id: Option<Uuid>,
        title: Option<&str>,
    ) -> Result<ChatRoom> {
        let id_str = id.to_string();
        let user_id_str = user_id.to_string();
        let store_id_str = store_id.map(|s| s.to_string());

        sqlx::query(
            r#"
            INSERT INTO chat_rooms (id, user_id, store_id, title)
            VALUES (?, ?, ?, ?)
            "#,
        )
            .bind(&id_str)
            .bind(&user_id_str)
            .bind(&store_id_str)
            .bind(title)
            .execute(&self.pool)
            .await?;

        self.find_room_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to fetch created chat room"))
    }

    pub async fn find_room_by_id(&self, id: Uuid) -> Result<Option<ChatRoom>> {
        let id_str = id.to_string();

        let room = sqlx::query_as::<_, ChatRoom>(
            r#"
            SELECT id, user_id, store_id, title, created_at, updated_at
            FROM chat_rooms
            WHERE id = ?
            "#,
        )
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await?;

        Ok(room)
    }

    pub async fn list_rooms_by_user_id(&self, user_id: Uuid) -> Result<Vec<ChatRoom>> {
        let user_id_str = user_id.to_string();

        let rooms = sqlx::query_as::<_, ChatRoom>(
            r#"
            SELECT id, user_id, store_id, title, created_at, updated_at
            FROM chat_rooms
            WHERE user_id = ?
            ORDER BY updated_at DESC, created_at DESC
            "#,
        )
            .bind(&user_id_str)
            .fetch_all(&self.pool)
            .await?;

        Ok(rooms)
    }

    pub async fn delete_room(&self, id: Uuid) -> Result<bool> {
        let id_str = id.to_string();

        let result = sqlx::query("DELETE FROM chat_rooms WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn create_message(
        &self,
        id: Uuid,
        room_id: Uuid,
        role: &str,
        content: &str,
        sources: Option<&str>,
        model: Option<&str>,
        latency_ms: Option<i32>,
        cached: bool,
        error_message: Option<&str>,
    ) -> Result<ChatMessage> {
        let id_str = id.to_string();
        let room_id_str = room_id.to_string();

        sqlx::query(
            r#"
            INSERT INTO chat_messages (
                id, room_id, role, content, sources,
                model, latency_ms, cached, error_message
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
            .bind(&id_str)
            .bind(&room_id_str)
            .bind(role)
            .bind(content)
            .bind(sources)
            .bind(model)
            .bind(latency_ms)
            .bind(cached)
            .bind(error_message)
            .execute(&self.pool)
            .await?;

        self.find_message_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to fetch created chat message"))
    }

    pub async fn find_message_by_id(&self, id: Uuid) -> Result<Option<ChatMessage>> {
        let id_str = id.to_string();

        let message = sqlx::query_as::<_, ChatMessage>(
            r#"
            SELECT id, room_id, role, content, sources,
                   model, latency_ms, cached, error_message, created_at
            FROM chat_messages
            WHERE id = ?
            "#,
        )
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await?;

        Ok(message)
    }

    pub async fn list_messages_by_room_id(&self, room_id: Uuid) -> Result<Vec<ChatMessage>> {
        let room_id_str = room_id.to_string();

        let messages = sqlx::query_as::<_, ChatMessage>(
            r#"
            SELECT id, room_id, role, content, sources,
                   model, latency_ms, cached, error_message, created_at
            FROM chat_messages
            WHERE room_id = ?
            ORDER BY created_at ASC
            "#,
        )
            .bind(&room_id_str)
            .fetch_all(&self.pool)
            .await?;

        Ok(messages)
    }

    pub async fn get_recent_messages(&self, room_id: Uuid, limit: i64) -> Result<Vec<ChatMessage>> {
        let room_id_str = room_id.to_string();

        let messages = sqlx::query_as::<_, ChatMessage>(
            r#"
            SELECT id, room_id, role, content, sources,
                   model, latency_ms, cached, error_message, created_at
            FROM (
                SELECT id, room_id, role, content, sources,
                       model, latency_ms, cached, error_message, created_at
                FROM chat_messages
                WHERE room_id = ?
                ORDER BY created_at DESC
                LIMIT ?
            )
            ORDER BY created_at ASC
            "#,
        )
            .bind(&room_id_str)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        Ok(messages)
    }
}