use serde::Serialize;
use serde_json::Value;

use crate::catalog::entities::product::Product;

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: String,
    pub store_id: String,
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub specs: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub faq: Option<Value>,

    pub status: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,

    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Product> for ProductResponse {
    fn from(product: Product) -> Self {
        let specs = product.specs.and_then(|s| serde_json::from_str(&s).ok());
        let faq = product.faq.and_then(|f| serde_json::from_str(&f).ok());

        Self {
            id: product.id,
            store_id: product.store_id,
            name: product.name,
            category: product.category,
            description: product.description,
            specs,
            manual: product.manual,
            faq,
            status: product.status,
            external_id: product.external_id,
            source: product.source,
            created_at: product.created_at,
            updated_at: product.updated_at,
        }
    }
}