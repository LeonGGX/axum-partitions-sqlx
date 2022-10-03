//! src/models/user.rs
//!

//use axum_login::AuthUser;
use serde::Serialize;
use uuid::Uuid;
//use sha3::digest::typenum::private::Trim;
use sqlx::FromRow;
use unicode_segmentation::UnicodeSegmentation;

///
/// User
/// struct to handle users
/// fields : id, name, password_hash, role
///
#[derive(Debug, Clone, Serialize, Default, FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub password_hash: String,
    pub role: String,
}

/*
impl AuthUser for User {
    fn get_id(&self) -> String {
        format!("{}", self.id)
    }

    fn get_password_hash(&self) -> String {
        self.password_hash.clone()
    }
}
*/
///
/// NewUser
///
/// Struct to handle new user
/// Uses NewUserName to protect name integrity
/// in struct NewUser
///
#[derive(Debug)]
pub struct NewUser {
    pub name: NewUserName,
    pub password: String,
    pub role: String,
}

//impl Sized for NewUser {}

///
/// struct to protect user_name integrity in struct User
///
#[derive(Debug)]
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
    #[allow(dead_code)]
    pub fn parse(s: String) -> Result<NewUserName, String> {
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
            return Err(format!("{} is not a valid new user name.", s));
            //panic!("{} is not a valid new user name.", s)
        } else {
            return Ok(Self(s));
        }
    }
}
