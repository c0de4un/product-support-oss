CREATE TABLE IF NOT EXISTS stores (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    domain TEXT UNIQUE,
    description TEXT,
    api_token TEXT UNIQUE,
    status TEXT NOT NULL DEFAULT 'active',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME
);

CREATE INDEX IF NOT EXISTS idx_stores_user_id ON stores(user_id);
CREATE INDEX IF NOT EXISTS idx_stores_domain ON stores(domain);
CREATE UNIQUE INDEX IF NOT EXISTS idx_stores_api_token ON stores(api_token);