use serde::Serialize;

use crate::store::entities::store::Store;

#[derive(Debug, Serialize)]
pub struct StoreResponse {
    pub id: String,
    pub user_id: String,
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_token: Option<String>,

    pub status: String,
    pub created_at: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

impl StoreResponse {
    pub fn public(store: Store) -> Self {
        Self {
            id: store.id,
            user_id: store.user_id,
            name: store.name,
            domain: store.domain,
            description: store.description,
            api_token: None,
            status: store.status,
            created_at: store.created_at,
            updated_at: store.updated_at,
        }
    }

    pub fn with_api_token(store: Store) -> Self {
        let api_token = store.api_token.clone();
        let mut response = Self::public(store);
        response.api_token = api_token;
        response
    }
}

impl From<Store> for StoreResponse {
    fn from(store: Store) -> Self {
        StoreResponse::public(store)
    }
}