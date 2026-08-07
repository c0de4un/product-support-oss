use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateStoreRequest {
    #[validate(length(min = 1, max = 255, message = "Store name is required"))]
    pub name: String,

    #[validate(length(max = 255, message = "Domain is too long"))]
    pub domain: Option<String>,

    #[validate(length(max = 2000, message = "Description is too long"))]
    pub description: Option<String>,
}