//! src/models/musician
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// this struct will be used to represent database record
#[derive(Clone, Serialize, Deserialize, FromRow, Debug, Eq, PartialEq)]
pub struct Person {
    #[serde(skip_deserializing)]
    pub id: Option<i32>,
    pub full_name: String,
}
