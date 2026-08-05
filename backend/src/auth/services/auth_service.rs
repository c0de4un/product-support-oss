use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, errors::Error};
use uuid::Uuid;
use crate::config::config::Config;
use crate::auth::domain::claims::Claims;

pub fn create_jwt(user_id: &Uuid, config: &Config) -> Result<String, Error> {
    let now = Utc::now();
    let exp = now + Duration::hours(config.jwt_expires_hours);

    let claims = Claims {
        sub: user_id.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_ref()),
    )
}

pub fn verify_jwt(token: &str, config: &Config) -> Result<Claims, Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}