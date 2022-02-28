//! src/models.rs
//!
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

/*
#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct InsertablePerson {
    pub full_name: String,
}

impl FromStr for InsertablePerson {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(InsertablePerson {
            full_name: "".to_string(),
        })
    }
}
*/

// this struct will be used to represent database record
#[derive(Clone, Serialize, Deserialize, FromRow, Debug, Eq, PartialEq)]
pub struct Person {
    #[serde(skip_deserializing)]
    pub id: Option<i32>,
    pub full_name: String,
}



#[derive(
Debug,
Clone,
Deserialize,
Serialize,
PartialEq,
FromRow,
)]
pub struct Genre {
    #[serde(skip_deserializing)]
    pub id: Option<i32>,
    pub name: String,
}

#[derive(
Debug, Clone, Deserialize, Serialize, FromRow,
)]
pub struct Partition {
    #[serde(skip_deserializing)]
    pub id: Option<i32>,
    pub title: String,
    pub person_id: i32,
    pub genre_id: i32,
}

///
/// une struct pour présenter les partitions avec les
/// éléments des différentes tables
///
#[derive(Debug, Serialize, Deserialize)]
pub struct ShowPartition {
    #[serde(skip_deserializing)]
    pub id: Option<i32>,
    pub title: String,
    pub full_name: String,
    pub name: String,
}

///
/// User
/// struct to handle users
/// fields : user_id, user_name, password_hash, role
///
#[derive(Debug, Clone)]
pub struct User {
    pub user_id: Uuid,
    pub user_name: String,
    pub password_hash: String,
    pub role: String,
}

///
/// NewUser
///
/// Struct to handle new user
/// Uses UserName to protect user_name integrity
/// in struct NewUser
///
#[derive(Debug,)]
pub struct NewUser {
    pub user_name: NewUserName,
    pub password: String,
    pub role: String,
}

///
/// struct to protect user_name integrity in struct User
///
#[derive(Debug,)]
pub struct NewUserName(String);

impl AsRef<str> for NewUserName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl NewUserName {
    /// Returns an instance of `UserName` if the input satisfies all
    /// our validation constraints on subscriber names.
    /// It panics otherwise.
    pub fn parse(s: String) -> NewUserName {
        // `.trim()` returns a view over the input `s` without trailing
        // whitespace-like characters.
        // `.is_empty` checks if the view contains any character.
        let is_empty_or_whitespace = s.trim().is_empty();

        // A grapheme is defined by the Unicode standard as a "user-perceived"
        // character: `å` is a single grapheme, but it is composed of two characters
        // (`a` and `̊`).
        //
        // `graphemes` returns an iterator over the graphemes in the input `s`.
        // `true` specifies that we want to use the extended grapheme definition set,
        // the recommended one.
        let is_too_long = s.graphemes(true).count() > 256;

        // Iterate over all characters in the input `s` to check if any of them matches
        // one of the characters in the forbidden array.
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            panic!("{} is not a valid new user name.", s)
        } else {
            Self(s)
        }
    }
}




