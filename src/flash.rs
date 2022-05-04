//! src/flash.rs

use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Redirect};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};
use axum_flash::{Flash,};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlashData {
    pub(crate) kind: String,
    pub(crate) message: String,
}

#[derive(Deserialize)]
struct ValuedMessage<T> {
    #[serde(rename = "_")]
    value: T,
}

#[derive(Serialize)]
struct ValuedMessageRef<'a, T> {
    #[serde(rename = "_")]
    value: &'a T,
}

const FLASH_COOKIE_NAME: &str = "_flash";

pub fn get_flash_cookie<T>(cookies: &Cookies) -> Option<T>
    where
        T: DeserializeOwned,
{
    cookies.get(FLASH_COOKIE_NAME).and_then(|flash_cookie| {
        if let Ok(ValuedMessage::<T> { value }) = serde_json::from_str(flash_cookie.value()) {
            Some(value)
        } else {
            None
        }
    })
}

pub type PersonResponse = (StatusCode, HeaderMap);
pub type GenreResponse = (StatusCode, HeaderMap);
pub type PartitionResponse = (StatusCode, HeaderMap);
//pub type UserResponse = (StatusCode, HeaderMap);
pub type SignupResponse = (StatusCode, HeaderMap);
pub type LoginResponse = (StatusCode, HeaderMap);

/*
pub fn person_response<T>(cookies: &mut Cookies, data: T) -> PersonResponse
    where
        T: Serialize,
{
    let valued_message_ref = ValuedMessageRef { value: &data };

    let mut cookie = Cookie::new(
        FLASH_COOKIE_NAME,
        serde_json::to_string(&valued_message_ref).unwrap(),
    );
    cookie.set_path("/persons");
    cookies.add(cookie);

    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static("/persons"));

    (StatusCode::SEE_OTHER, header)
}
*/
/*
pub fn genre_response<T>(cookies: &mut Cookies, data: T) -> GenreResponse
    where
        T: Serialize,
{
    let valued_message_ref = ValuedMessageRef { value: &data };

    let mut cookie = Cookie::new(
        FLASH_COOKIE_NAME,
        serde_json::to_string(&valued_message_ref).unwrap(),
    );
    cookie.set_path("/genres");
    cookies.add(cookie);

    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static("/genres"));

    (StatusCode::SEE_OTHER, header)
}
*/
/*
pub fn partition_response<T>(cookies: &mut Cookies, data: T) -> PartitionResponse
    where
        T: Serialize,
{
    let valued_message_ref = ValuedMessageRef { value: &data };

    let mut cookie = Cookie::new(
        FLASH_COOKIE_NAME,
        serde_json::to_string(&valued_message_ref).unwrap(),
    );
    cookie.set_path("/partitions");
    cookies.add(cookie);

    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static("/partitions"));

    (StatusCode::SEE_OTHER, header)
}
*/
/*
pub fn user_response<T>(cookies: &mut Cookies, data: T) -> UserResponse
    where
        T: Serialize,
{
    let valued_message_ref = ValuedMessageRef { value: &data };

    let mut cookie = Cookie::new(
        FLASH_COOKIE_NAME,
        serde_json::to_string(&valued_message_ref).unwrap(),
    );
    cookie.set_path("/auth/signup");
    cookies.add(cookie);

    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static("/auth/signup"));

    (StatusCode::SEE_OTHER, header)
}
 */

pub fn person_response(flash: &mut Flash, message: String) -> PersonResponse {
    flash.info(message);
    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static("/persons"));
    //Redirect::to("/auth/signup")
    (StatusCode::SEE_OTHER, header)
}

pub fn genre_response(flash: &mut Flash, message: String) -> GenreResponse {
    flash.info(message);
    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static("/genres"));
    //Redirect::to("/auth/signup")
    (StatusCode::SEE_OTHER, header)
}

pub fn partition_response(flash: &mut Flash, message: String) -> PartitionResponse {
    flash.info(message);
    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static("/partitions"));

    (StatusCode::SEE_OTHER, header)
}

pub fn signup_response(flash: &mut Flash, message: String) -> SignupResponse {
    flash.info(message);
    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static("/auth/signup"));
    //Redirect::to("/auth/signup")
    (StatusCode::SEE_OTHER, header)
}

pub fn login_response(flash: &mut Flash, message: String) -> LoginResponse {
    flash.info(message);
    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static("/auth/login"));
    //Redirect::to("/auth/signup")
    (StatusCode::SEE_OTHER, header)
}

pub fn user_response(flash: &mut Flash, message: String) -> impl IntoResponse {
    flash.info(message);
    Redirect::to("/auth/signup")
}