use serde::Deserialize;
use serde_json::Value;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProductRequest {
    #[validate(length(min = 1, max = 255, message = "Product name is required"))]
    pub name: String,

    #[validate(length(max = 255, message = "Category is too long"))]
    pub category: Option<String>,

    pub description: Option<String>,

    /// JSON object with technical specifications
    /// Example: {"material": "cotton", "weight": "250g"}
    pub specs: Option<Value>,

    pub manual: Option<String>,

    /// JSON array of FAQ objects
    /// Example: [{"question": "Can I return?", "answer": "Yes"}]
    pub faq: Option<Value>,

    #[validate(length(max = 255, message = "External ID is too long"))]
    pub external_id: Option<String>,

    #[validate(length(max = 50, message = "Source is too long"))]
    pub source: Option<String>,
}