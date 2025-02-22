use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Validation};
use chrono::{Utc, Duration};
use serde::{Serialize, Deserialize};
use jsonwebtoken::errors::Error;

const SECRET_KEY: &[u8] = b"your_secret_key";  

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthTokenClaims {  
    pub sub: String, 
    pub exp: usize,  
    pub token_type: String, 
}

fn generate_token(email: &str, expiry_minutes: i64, token_type: &str, secret: &[u8]) -> Result<String, Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::minutes(expiry_minutes))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = AuthTokenClaims {  
        sub: email.to_owned(),
        exp: expiration,
        token_type: token_type.to_string(),
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
}

/// Generates an access token that expires in **7 minutes**
pub fn generate_access_token(email: &str) -> Result<String, Error> {
    generate_token(email, 7, "access", SECRET_KEY) // 7 minutes expiry
}

/// Generates a refresh token that expires in **15 days**
pub fn generate_refresh_token(email: &str) -> Result<String, Error> {
    generate_token(email, 15 * 24 * 60, "refresh", SECRET_KEY) // 15 days expiry
}

impl AuthTokenClaims {
    pub fn validate_token(token: &str) -> Result<Self, Error> {
        let token_data = decode::<AuthTokenClaims>(
            token,
            &DecodingKey::from_secret(SECRET_KEY),  
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }
}
