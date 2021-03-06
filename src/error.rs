// src/error.rs

use axum::http::header::WWW_AUTHENTICATE;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse,};
use axum::Json;

use sqlx::error::DatabaseError;

use thiserror::Error;

use std::borrow::Cow;
use std::collections::HashMap;
use serde_json::{json,};

#[derive(Debug, Error)]
pub enum MyError {
    //#[error(transparent)]
    //BcryptError(#[from] bcrypt::BcryptError),
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    TokioRecvError(#[from] tokio::sync::oneshot::error::RecvError),
    #[error(transparent)]
    AxumTypedHeaderError(#[from] axum::extract::rejection::TypedHeaderRejection),
    #[error(transparent)]
    AxumExtensionError(#[from] axum::extract::rejection::ExtensionRejection),
    //#[error(transparent)]
    //ValidationError(#[from] validator::ValidationErrors),
    #[error("wrong credentials")]
    WrongCredentials,
    #[error("password doesn't match")]
    WrongPassword,
    #[error("email is already taken")]
    DuplicateUserEmail,
    #[error("name is already taken")]
    DuplicateUserName,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

/*
//pub type Result<T> = std::result::Result<T, MyError>;
pub type ApiError = (StatusCode, Json<Value>);

impl From<MyError> for ApiError {
    fn from(err: MyError) -> Self {
        let status = match err {
            MyError::WrongCredentials => StatusCode::UNAUTHORIZED,
            //MyError::ValidationError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let payload = json!({"message": err.to_string()});
        (status, Json(payload))
    }
}
*/

#[derive(Debug, Error)]
pub enum AppError {
    /// Return `401 Unauthorized`
    #[error("authentication required")]
    Unauthorized,

    /// Return `403 Forbidden`
    #[error("user may not perform that action")]
    Forbidden,

    /// Return `404 Not Found`
    #[error("request path not found")]
    NotFound,

    #[error("error in the request body")]
    UnprocessableEntity {
        errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
    },

    #[error(transparent)]
    Sqlx( #[from] sqlx::Error),

    /// Via the generated `From<anyhow::Error> for Error` impl, this allows the
    /// use of `?` in handler functions to automatically convert `anyhow::Error` into a response.
    ///
    /// Like with `Error::Sqlx`, the actual error message is not returned to the client
    /// for security reasons.
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error(transparent)]
    Tera(#[from] tera::Error),

    // ce qui suit : rajout?? pour le syst??me d'autentification JWT
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

    #[error("no permission")]
    NoPermissionError,

    #[error("Validations error")]
    ValidationError,
/*
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
*/
    #[error("User already exists")]
    UserExists,

}

impl AppError {

    /// Convenient constructor for `Error::UnprocessableEntity`.
    ///
    /// Multiple for the same key are collected into a list for that key.
    ///
    /// Try "Go to Usage" in an IDE for examples.
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
            Self::NoPermissionError => StatusCode::UNAUTHORIZED,
            Self::ValidationError => StatusCode::INTERNAL_SERVER_ERROR,
            //Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UserExists => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Axum allows you to return `Result` from handler functions, but the error type
/// also must be some sort of response type.
///
/// By default, the generated `Display` impl is used to return a plaintext error message
/// to the client.
impl IntoResponse for AppError {
    //type Body = Full<Bytes>;
    //type BodyError = <Full<Bytes> as HttpBody>::Error;

    //fn into_response(self) -> Response<Self::Body> {
    //fn into_response(self) -> Response<Full<Bytes>> {
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
            }

            Self::Anyhow(ref e) => {
                // TODO: we probably want to use `tracing` instead
                // so that this gets linked to the HTTP request by `TraceLayer`.
                tracing::error!("Generic error: {:?}", e);
                return (
                    self.status_code(),
                    Json(json!({
                        "message :" : e.to_string()
                    }))
                ).into_response()
            }

            Self::Tera(ref e) => {
                tracing::error!("Tera error : {:?}", e);
                let body =  Json(json!({
                        "message :" : e.to_string()
                    }));
                return (
                    self.status_code(),
                    body
                ).into_response()
            }

            Self::JWTTokenCreationError(ref e) => {
                tracing::error!("Token creation error : {:?}", e);
                let error_json = json!({"message" : e.to_string()});
                let body = Json(json!(
                    error_json
                ));
                return (
                    self.status_code(),
                    body
                ).into_response()
            }

            // Other errors get mapped normally.
            _ => (),
        }
        (self.status_code(), self.to_string()).into_response()
    }
}

/*
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
*/

pub trait ResultExt<T> {
    /// If `self` contains a SQLx database constraint error with the given name,
    /// transform the error.
    ///
    /// Otherwise, the result is passed through unchanged.
    fn on_constraint(
        self,
        name: &str,
        f: impl FnOnce(Box<dyn DatabaseError>) -> AppError,
    ) -> Result<T, AppError>;
}

impl<T, E> ResultExt<T> for Result<T, E>
    where
        E: Into<AppError>,
{
    fn on_constraint(
        self,
        name: &str,
        map_err: impl FnOnce(Box<dyn DatabaseError>) -> AppError,
    ) -> Result<T, AppError> {
        self.map_err(|e| match e.into() {
            AppError::Sqlx(sqlx::Error::Database(dbe)) if dbe.constraint() == Some(name) => {
                map_err(dbe)
            }
            e => e,
        })
    }
}
