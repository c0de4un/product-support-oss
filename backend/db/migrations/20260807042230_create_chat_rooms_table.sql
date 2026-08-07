CREATE TABLE IF NOT EXISTS chat_rooms (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    store_id TEXT,
    title TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME
);

CREATE INDEX IF NOT EXISTS idx_chat_rooms_user_id ON chat_rooms(user_id);
CREATE INDEX IF NOT EXISTS idx_chat_rooms_store_id ON chat_rooms(store_id);