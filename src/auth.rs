use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use chrono::{Utc, Duration};

const SECRET_KEY: &[u8] = b"your-secret-key"; // Replace with a secure key

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthTokenClaims {
    pub sub: String,
    pub exp: usize,
}

impl AuthTokenClaims {
    /// Creates a new token claim
    pub fn new(user_id: i32) -> Self {
        let expiration = Utc::now() + Duration::hours(24); // Token expires in 24 hours
        Self {
            sub: user_id.to_string(),
            exp: expiration.timestamp() as usize,
        }
    }

    /// Generates a JWT token
    pub fn generate_token(&self) -> Result<String, jsonwebtoken::errors::Error> {
        encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(SECRET_KEY),
        )
    }

    /// Validates a JWT token
    pub fn validate_token(token: &str) -> Result<AuthTokenClaims, jsonwebtoken::errors::Error> {
        let token_data = decode::<AuthTokenClaims>(
            token,
            &DecodingKey::from_secret(SECRET_KEY),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }
}
