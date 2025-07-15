use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};

pub fn generate_token(user_id: &str, secret: &str) -> String {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(1))
        .unwrap()
        .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .expect("Token creation failed")
}

use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::models::Claims;

pub fn verify_token(token: &str, secret: &str) -> Option<String> {
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .ok()?;

    Some(decoded.claims.sub)
}
