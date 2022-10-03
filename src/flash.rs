//! src/flash.rs
//!
//! Fonctions réponses qui renvoient un message flash sur la page d'origine
//! après une manipulation sur la base de données

use axum::http;
use axum::http::{header, HeaderMap, HeaderValue, Response, StatusCode};
use axum::response::{IntoResponse, Redirect};
use axum_flash::Flash;

///
/// retourne un message flash sur la page '/persons' (liste des musiciens)
/// après une action sur les musiciens
///
pub fn person_response(
    flash: &mut Flash,
    level: axum_flash::Level,
    message: String,
) -> (StatusCode, HeaderMap) {
    flash.push(level, message);
    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static("/persons"));

    (StatusCode::SEE_OTHER, header)
}

pub fn genre_response(flash: &mut Flash, message: String) -> (StatusCode, HeaderMap) {
    flash.info(message);
    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static("/genres"));

    (StatusCode::SEE_OTHER, header)
}

pub fn partition_response(
    flash: &mut Flash,
    level: axum_flash::Level,
    message: String,
) -> (StatusCode, HeaderMap) {
    flash.push(level, message);
    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static("/partitions"));

    (StatusCode::SEE_OTHER, header)
}

pub fn signup_response(flash: &mut Flash, level: axum_flash::Level, message: String) -> Redirect {
    flash.push(level, message);
    /*    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static("/auth/signup"));

    (StatusCode::SEE_OTHER, header)*/
    Redirect::to("/auth/signup")
}

pub fn login_response(flash: &mut Flash, level: axum_flash::Level, message: String) -> Redirect {
    flash.push(level, message);
    Redirect::to("/auth/login")
}

#[allow(dead_code)]
pub(crate) fn error_page(err: &dyn std::error::Error) -> impl IntoResponse {
    Response::builder()
        .status(http::StatusCode::INTERNAL_SERVER_ERROR)
        .body(format!("Err: {}", err))
        .unwrap()
}
