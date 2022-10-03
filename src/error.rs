// src/error.rs

use axum::http::header::WWW_AUTHENTICATE;
use axum::http::{HeaderMap, HeaderValue, Response, StatusCode};
use axum::response::IntoResponse;
use axum::Json;

use thiserror::Error;

use serde_json::json;
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;


#[derive(Debug, Error)]
pub enum AppError {
    /// Return `401 Unauthorized`
    #[error("Il faut s'identifier")]
    Unauthorized,

    /// Return `403 Forbidden`
    #[error("l'Utilisateur ne peut faire cette action")]
    Forbidden,

    /// Return `404 Not Found`
    #[error("Page non trouvée")]
    NotFound,

    #[error("error in the request body")]
    UnprocessableEntity {
        errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
    },

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    /// Via the generated `From<anyhow::Error> for Error` impl, this allows the
    /// use of `?` in handler functions to automatically convert `anyhow::Error` into a response.
    ///
    /// Like with `Error::Sqlx`, the actual error message is not returned to the client
    /// for security reasons.
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error(transparent)]
    Tera(#[from] tera::Error),

    // ce qui suit : rajouté pour le système d'autentification JWT
    #[error("wrong credentials")]
    WrongCredentials(#[source] anyhow::Error),

    #[error("missing credentials")]
    MissingCredentials,

    #[error("jwt token not valid")]
    InvalidJWTToken,

    #[error("jwt token creation error")]
    JWTTokenCreationError(#[from] jsonwebtoken::errors::Error),

    #[error("no auth header")]
    NoAuthHeaderError,

    #[error("invalid auth header")]
    InvalidAuthHeaderError,
    /*
        #[error("no permission")]
        NoPermissionError,
    */
    #[error("Validations error")]
    ValidationError,
    /*
        #[error(transparent)]
        UnexpectedError(#[from] anyhow::Error),

        #[error("User already exists")]
        UserExists,
    */
}

impl AppError {
    /// Convenient constructor for `Error::UnprocessableEntity`.
    ///
    /// Multiple for the same key are collected into a list for that key.
    ///
    /// Try "Go to Usage" in an IDE for examples.
    ///
    #[allow(dead_code)]
    pub fn unprocessable_entity<K, V>(errors: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        let mut error_map = HashMap::new();

        for (key, val) in errors {
            error_map
                .entry(key.into())
                .or_insert_with(Vec::new)
                .push(val.into());
        }

        Self::UnprocessableEntity { errors: error_map }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Sqlx(_) | Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Tera(_) => StatusCode::INTERNAL_SERVER_ERROR,

            Self::WrongCredentials(_) => StatusCode::UNAUTHORIZED,
            Self::MissingCredentials => StatusCode::BAD_REQUEST,
            Self::JWTTokenCreationError(_) => StatusCode::UNAUTHORIZED,
            Self::InvalidJWTToken => StatusCode::UNAUTHORIZED,
            Self::NoAuthHeaderError => StatusCode::BAD_REQUEST,
            Self::InvalidAuthHeaderError => StatusCode::BAD_REQUEST,
            //Self::NoPermissionError => StatusCode::UNAUTHORIZED,
            Self::ValidationError => StatusCode::INTERNAL_SERVER_ERROR,
            //Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            //Self::UserExists => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Axum allows you to return `Result` from handler functions, but the error type
/// also must be some sort of response type.
///
/// By default, the generated `Display` impl is used to return a plaintext error message
/// to the client.
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::UnprocessableEntity { errors } => {
                #[derive(serde::Serialize)]
                struct Errors {
                    errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
                }

                return (StatusCode::UNPROCESSABLE_ENTITY, Json(Errors { errors })).into_response();
            }
            Self::Unauthorized => {
                return (
                    self.status_code(),
                    // Include the `WWW-Authenticate` challenge required in the specification
                    // for the `401 Unauthorized` response code:
                    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401
                    //
                    // The Realworld spec does not specify this:
                    // https://realworld-docs.netlify.app/docs/specs/backend-specs/error-handling
                    //
                    // However, at Launchbadge we try to adhere to web standards wherever possible,
                    // if nothing else than to try to act as a vanguard of sanity on the web.
                    [(WWW_AUTHENTICATE, HeaderValue::from_static("Token"))]
                        .into_iter()
                        .collect::<HeaderMap>(),
                    self.to_string(),
                )
                    .into_response();
            }

            Self::Sqlx(ref e) => {
                // TODO: we probably want to use `tracing` instead
                // so that this gets linked to the HTTP request by `TraceLayer`.
                tracing::error!("SQLx error: {:?}", e);
                let body = Json(json!({
                    "message :" : e.to_string()
                }));
                return (self.status_code(), body).into_response();
            }

            Self::Anyhow(ref e) => {
                // TODO: we probably want to use `tracing` instead
                // so that this gets linked to the HTTP request by `TraceLayer`.
                tracing::error!("Generic error: {:?}", e);
                return (
                    self.status_code(),
                    Json(json!({
                        "message :" : e.to_string()
                    })),
                )
                    .into_response();
            }

            Self::Tera(ref e) => {
                tracing::error!("Tera error : {:?}", e);
                let body = Json(json!({
                    "message :" : e.to_string()
                }));
                return (self.status_code(), body).into_response();
            }

            Self::JWTTokenCreationError(ref e) => {
                tracing::error!("Token creation error : {:?}", e);
                let error_json = json!({"message" : e.to_string()});
                let body = Json(json!(error_json));
                return (self.status_code(), body).into_response();
            }

            // Other errors get mapped normally.
            _ => (),
        }
        (self.status_code(), self.to_string()).into_response()
    }
}


#[derive(Debug)]
pub enum SignupError {
    UsernameExists,
    InvalidUsername,
    PasswordsDoNotMatch,
    MissingPassword,
    MissingUserName,
    MissingPwConfirm,
    MissingRole,
    InvalidPassword,
    InternalError,
}

impl Display for SignupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignupError::InvalidUsername => f.write_str("Nom d'utilisateur incorrect"),
            SignupError::UsernameExists => f.write_str("Cet Utilisateur existe déjà"),
            SignupError::PasswordsDoNotMatch => f.write_str("Mot de passe non confirmé"),
            SignupError::MissingPassword => f.write_str("Il faut entrer un mot de passe"),
            SignupError::MissingUserName => f.write_str("Il faut entrer un nom d'utilisateur"),
            SignupError::MissingPwConfirm => f.write_str("Il faut confirmer le mot de passe"),
            SignupError::MissingRole => f.write_str("Il faut entrer un rôle"),
            SignupError::InvalidPassword => f.write_str("Mot de passe incorrect"),
            SignupError::InternalError => f.write_str("Erreur Serveur"),
        }
    }
}

impl Error for SignupError {}

impl IntoResponse for SignupError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            SignupError::InvalidUsername => {
                (StatusCode::BAD_REQUEST, "Nom d'utilisateur incorrect")
            }
            SignupError::UsernameExists => {
                (StatusCode::UNAUTHORIZED, "Cet Utilisateur existe déjà")
            }
            SignupError::PasswordsDoNotMatch => {
                (StatusCode::UNAUTHORIZED, "Mot de passe non confirmé")
            }
            SignupError::MissingPassword => {
                (StatusCode::BAD_REQUEST, "Il faut entrer un mot de passe")
            }
            SignupError::MissingUserName => (
                StatusCode::BAD_REQUEST,
                "Il faut entrer un nom d'utilisateur",
            ),
            SignupError::MissingPwConfirm => {
                (StatusCode::BAD_REQUEST, "Il faut confirmer le mot de passe")
            }
            SignupError::MissingRole => (StatusCode::BAD_REQUEST, "Il faut entrer un rôle"),
            SignupError::InvalidPassword => (StatusCode::UNAUTHORIZED, "Mot de passe incorrect"),
            SignupError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Erreur Serveur"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

#[derive(Debug)]
pub(crate) enum LoginError {
    MissingPassword,
    MissingUserName,
    UserDoesNotExist,
    WrongPassword,
}

impl Display for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginError::UserDoesNotExist => f.write_str("Cet Utilisateur n'existe pas"),
            LoginError::MissingPassword => f.write_str("Il faut entrer un Mot de Passe"),
            LoginError::MissingUserName => f.write_str("Il faut entrer le Nom d'Utilisateur"),
            LoginError::WrongPassword => f.write_str("Mot de passe incorrect"),
        }
    }
}

impl Error for LoginError {}
