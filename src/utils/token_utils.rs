use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, DecodingKey, EncodingKey, Validation};

use crate::models::token_models::Claims;

pub fn generate_jwt(user_id: &str, secret: &[u8]) -> String {
    let expiration = Utc::now() + Duration::hours(720); // 30 days
    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration.timestamp(),
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret)).unwrap()
}

pub fn verify_jwt(token: &str, secret: &[u8]) -> Option<Claims> {
    decode::<Claims>(token, &DecodingKey::from_secret(secret), &Validation::default())
        .ok()
        .map(|data| data.claims)
}