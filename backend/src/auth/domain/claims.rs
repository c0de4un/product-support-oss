use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // subject (user_id)
    pub exp: usize,  // expiration time (UNIX timestamp)
    pub iat: usize,  // issued at (UNIX timestamp)
}