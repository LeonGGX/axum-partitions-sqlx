//! src/flash.rs
//!
//! Fonctions réponses qui renvoient un message flash sur la page d'origine
//! après une manipulation sur la base de données

use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Redirect};
use axum_flash::Flash;

///
/// retourne un message flash sur la page '/persons' (liste des musiciens)
/// après une action sur les musiciens
///
pub fn person_response(flash: &mut Flash, message: String) -> (StatusCode, HeaderMap) {
    flash.info(message);
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

pub fn partition_response(flash: &mut Flash, message: String) -> (StatusCode, HeaderMap) {
    flash.info(message);
    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static("/partitions"));

    (StatusCode::SEE_OTHER, header)
}

pub fn signup_response(flash: &mut Flash, message: String) -> (StatusCode, HeaderMap) {
    flash.info(message);
    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static("/auth/signup"));

    (StatusCode::SEE_OTHER, header)
}

pub fn login_response(flash: &mut Flash, message: String) -> (StatusCode, HeaderMap) {
    flash.info(message);
    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static("/auth/login"));

    (StatusCode::SEE_OTHER, header)
}
#[allow(dead_code)]
pub fn user_response(flash: &mut Flash, message: String) -> impl IntoResponse {
    flash.info(message);
    Redirect::to("/auth/signup")
}