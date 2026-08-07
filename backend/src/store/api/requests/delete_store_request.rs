use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DeleteStoreRequest {
    pub confirm: Option<bool>,
}