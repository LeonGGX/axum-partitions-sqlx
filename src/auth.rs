//! src/auth.rs

use std::fmt::Display;
use chrono::{Duration, Utc};
use anyhow::{anyhow, Context};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash};
use once_cell::sync::Lazy;
use axum::{
    extract::{FromRequest, RequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    async_trait,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::error::AppError;

///
/// Permet d'encoder et décoder les tokens
/// 
static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    pub(crate) fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuthBody {
    pub access_token: String,
    pub token_type: String,
}

impl AuthBody {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SignInPayload {
    pub username: String,
    pub password: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub username: String,
    pub exp: i64,
}

impl Claims {
    pub fn new(id: Uuid) -> Self {
        let username = "".to_string();
        let iat = Utc::now();
        let exp = iat + Duration::hours(24);

        Self {
            sub: id,
            username,
            exp: exp.timestamp(),
        }
    }
}

impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "user_id: {}\nUserName: {}\nExp : {}\n", self.sub, self.username, self.exp)
    }
}

/// defines how to extract the claims from the request
#[async_trait]
impl<B> FromRequest<B> for Claims
    where
        B: Send,
{
    type Rejection = AppError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|_| AppError::InvalidJWTToken)?;
        // Decode the user data
        let token_data = decode::<Claims>(
            bearer.token(),
            &KEYS.decoding,
            &Validation::default()
        )
            .map_err(|_| AppError::InvalidJWTToken)?;

        Ok(token_data.claims)
    }
}

///
/// Utility function to hash passwords
/// Returns a String with hashed password or anyhow::Error
///
pub async fn hash_password(password: String) -> Result<String, anyhow::Error> {
    // Argon2 hashing is designed to be computationally intensive,
    // so we need to do this on a blocking thread.
    Ok(tokio::task::spawn_blocking(move || -> Result<String, anyhow::Error> {
        let salt = SaltString::generate(rand::thread_rng());
        Ok(
            PasswordHash::generate(Argon2::default(), password, salt.as_str())
                .map_err(|e| anyhow::anyhow!("failed to generate password hash: {}", e))?
                .to_string(),
        )
    })
        .await
        .context("panic in generating password hash")??)
}

///
/// Function verify_password
/// 1) password to verify
/// 2) stored password
///
/// uses argon2
///
pub async fn verify_password(entered_password: String, stored_password_hash: String) -> Result<(), anyhow::Error> {
    Ok(tokio::task::spawn_blocking(move || -> Result<(), anyhow::Error> {
        let hash = PasswordHash::new(&stored_password_hash)
            .map_err(|e| anyhow::anyhow!("invalid password hash: {}", e))?;

        hash.verify_password(&[&Argon2::default()], entered_password)
            .map_err(|e| match e {
                argon2::password_hash::Error::Password => anyhow!("Mot de passe inexact").into(),
                _ => anyhow::anyhow!("failed to verify password hash: {}", e).into(),
            })
    })
        .await
        .context("panic in verifying password hash")??)
}


pub fn generate_jwt(claims: &Claims) -> anyhow::Result<String> {
    encode(
        &Header::default(),
        &claims,
        //&EncodingKey::from_secret(JWT_SECRET.as_bytes()),
        &KEYS.encoding
    )
        .map_err(|e| anyhow::anyhow!(e))
}

pub fn sign_jwt(id: Uuid) -> anyhow::Result<String> {
    Ok(jsonwebtoken::encode(
        &Header::default(),
        &Claims::new(id),
        //&EncodingKey::from_secret(JWT_SECRET.as_bytes()),
        &KEYS.encoding
    )?)
}

pub fn verify_jwt(token: &str) -> anyhow::Result<Claims> {
    jsonwebtoken::decode(
        token,
        //&DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &KEYS.decoding,
        &Validation::default(),
    )
        .map(|data| data.claims)
        .map_err(|e| anyhow::anyhow!(e))
}
