//! src/utils/auth_utils.rs

use anyhow::{anyhow, Context};
use argon2::{password_hash::SaltString, Argon2, PasswordHash};
use pbkdf2::password_hash::PasswordHasher;
use pbkdf2::Pbkdf2;
use rand_core::OsRng;

use crate::error::SignupError;
use crate::AppError;
use unicode_segmentation::UnicodeSegmentation;

///
/// Utility function to parse usernames
/// Returns a String with the parsed username or AppError
///
pub fn parse(s: &String) -> Result<String, AppError> {
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
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}', '#', '*', ' '];
    let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

    if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
        return Err(AppError::ValidationError);
    } else {
        return Ok(s.to_string());
    }
}

//***************************************************************************************
//ARGON 2

///
/// Utility function to hash passwords with Argon2
/// Returns a String with hashed password or anyhow::Error
///
#[allow(dead_code)]
pub async fn hash_password_argon2(password: String) -> Result<String, anyhow::Error> {
    // Argon2 hashing is designed to be computationally intensive,
    // so we need to do this on a blocking thread.
    Ok(
        tokio::task::spawn_blocking(move || -> Result<String, anyhow::Error> {
            let salt = SaltString::generate(rand::thread_rng());
            Ok(
                PasswordHash::generate(Argon2::default(), password, salt.as_str())
                    .map_err(|e| anyhow::anyhow!("failed to generate password hash: {}", e))?
                    .to_string(),
            )
        })
        .await
        .context("panic in generating password hash")??,
    )
}

///
/// Function verify_password
/// 1) password to verify
/// 2) stored password
///
/// uses argon2
///
#[allow(dead_code)]
pub async fn verify_password_argon2(
    entered_password: String,
    stored_password_hash: String,
) -> Result<(), anyhow::Error> {
    Ok(
        tokio::task::spawn_blocking(move || -> Result<(), anyhow::Error> {
            let hash = PasswordHash::new(&stored_password_hash)
                .map_err(|e| anyhow::anyhow!("invalid password hash: {}", e))?;

            hash.verify_password(&[&Argon2::default()], entered_password)
                .map_err(|e| match e {
                    argon2::password_hash::Error::Password => {
                        anyhow!("Mot de passe inexact").into()
                    }
                    _ => anyhow::anyhow!("failed to verify password hash: {}", e).into(),
                })
        })
        .await
        .context("panic in verifying password hash")??,
    )
}

// end ARGON2 ************************************************************************************

//    ********************************************************************************************
///
/// Utility function to hash passwords with pbkdf2
/// Returns a String with hashed password or anyhow::Error
///
#[allow(dead_code)]
pub async fn hash_password_pbkdf2(password: String) -> Result<String, SignupError> {
    let salt = SaltString::generate(&mut OsRng);

    // Hash password to PHC string ($pbkdf2-sha256$...)
    let password_hash = Pbkdf2.hash_password(password.as_bytes(), &salt);

    let hashed_password = if let Ok(password) = password_hash {
        password.to_string()
    } else {
        return Err(SignupError::InvalidPassword);
    };
    Ok(hashed_password)
}
