use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: String,
    pub store_id: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,

    pub specs: Option<String>,

    pub manual: Option<String>,

    pub faq: Option<String>,

    pub status: String,
    pub external_id: Option<String>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}