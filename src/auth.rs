use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use chrono::{Utc, Duration};

const SECRET_KEY: &[u8] = b"your-secret-key"; 

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthTokenClaims {
    pub sub: i32,
    pub exp: usize,
}

impl AuthTokenClaims {
    pub fn new(user_id: i32, duration_hours: i64) -> Self {
        let expiration = Utc::now() + Duration::hours(duration_hours);
        Self {
            sub: user_id,
            exp: expiration.timestamp() as usize,
        }
    }

    pub fn generate_token(&self) -> Result<String, jsonwebtoken::errors::Error> {
        encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(SECRET_KEY),
        )
    }

    pub fn validate_token(token: &str) -> Result<AuthTokenClaims, jsonwebtoken::errors::Error> {
        decode::<AuthTokenClaims>(
            token,
            &DecodingKey::from_secret(SECRET_KEY),
            &Validation::default(),
        )
        .map(|token_data| token_data.claims)
    }
}
