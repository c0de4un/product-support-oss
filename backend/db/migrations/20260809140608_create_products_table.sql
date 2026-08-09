CREATE TABLE IF NOT EXISTS products (
    id TEXT PRIMARY KEY NOT NULL,
    store_id TEXT NOT NULL,
    name TEXT NOT NULL,
    category TEXT,
    description TEXT,
    specs TEXT,      -- JSON object (e.g. {"material": "cotton", "weight": "250g"})
    manual TEXT,
    faq TEXT,        -- JSON array of objects (e.g. [{"q": "...", "a": "..."}])
    status TEXT NOT NULL DEFAULT 'active',
    external_id TEXT,
    source TEXT NOT NULL DEFAULT 'manual',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (store_id) REFERENCES stores(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_products_store_id ON products(store_id);

CREATE INDEX IF NOT EXISTS idx_products_external_id ON products(external_id);