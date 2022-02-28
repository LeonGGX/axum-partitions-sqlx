//! src/handlers/login_hdl.rs

use axum::extract::{Extension, Form};
use axum::Json;
use jsonwebtoken::{encode, Header};
use sqlx::PgPool;

use crate::auth::{AuthBody, Claims, Keys, LoginPayload};
use crate::db::musicians::find_person_by_name;
use crate::error::AuthError;

use once_cell::sync::Lazy;

static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});


pub async fn login_hdl(
    Extension(ref pool): Extension<PgPool>,
    form: Form<LoginPayload>,) -> Result<Json<AuthBody>, AuthError> {

    if form.user_name.is_empty() || form.password.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    let users = find_person_by_name(form.user_name.clone(), pool).await;
    //let checked_user = users[0].clone();
    match users {
        Ok(users) => {
            let checked_user = users[0].clone();
            let claims = Claims {
                sub: "b@b.com".to_owned(),
                company: "ACME".to_owned(),
                exp: 100000,
            };
            // Create the authorization token
            let token = encode(&Header::default(), &claims, &KEYS.encoding)
                .map_err(|_| AuthError::TokenCreation)?;

            // Send the authorized token
            Ok(Json(AuthBody::new(token)))

        }
        Err(_) => return Err(AuthError::WrongCredentials),
    }
}