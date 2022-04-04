//! src/models.rs
//!
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;












#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct RegisterResponse {
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct LoginResponse {
    pub success: bool,
    pub token: String,
}


