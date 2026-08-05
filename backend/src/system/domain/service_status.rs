use serde::Serialize;

#[derive(Serialize)]
pub struct ServiceStatus {
    pub status: &'static str,
    pub detail: Option<String>,
}

impl ServiceStatus {
    pub fn pending() -> Self {
        Self {
            status: "pending",
            detail: Some("not initialized yet".to_string()),
        }
    }

    pub fn ok() -> Self {
        Self { status: "ok", detail: None }
    }
}