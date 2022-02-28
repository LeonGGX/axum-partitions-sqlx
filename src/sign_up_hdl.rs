//! src/handlers/sign_up_hdl.rs

use axum::extract::{Extension, Form};
use axum::response::Html;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use tera::Tera;
use tower_cookies::Cookies;

use crate::auth::SignInPayload;
use crate::{AppError,};
use crate::error::AuthError;
use crate::flash::{FlashData, user_response, UserResponse};
use crate::model::{NewUser, NewUserName,};
use crate::db::users::*;

#[derive(serde::Deserialize)]
pub struct FormData {
    user_name: String,
    password: String,
    role: String,
}

pub async fn get_sign_up_hdl(
    Extension(ref templates): Extension<Tera>,) -> Result<Html<String>, AppError> {

    let title = "Sign Up";
    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    let body = templates
        .render("sign_up.html.tera", &ctx)
        .map_err(|e| AppError::Tera(e))?;

    Ok(Html(body))
}

pub async fn sign_up_hdl(
    Extension(ref pool): Extension<PgPool>,
    form: Form<FormData>,
    mut cookies: Cookies,) -> Result<UserResponse, AppError> {

    if let Ok(user) = find_user_by_credentials(
        form.user_name.clone(), form.password.clone(), pool).await {

        tracing::info!("user already exists, new user not added : {:?}", user);
        let message =
            format!("Nouvel utilisateur pas ajouté, cet utilisateur existe déjà : {}", user.user_name);

        let data = FlashData {
            kind: "error".to_owned(),
            message: message.to_owned(),
        };
        Ok(user_response(&mut cookies, data))
    }
    else {
        let new_user = NewUser {
        user_name: NewUserName::parse(form.user_name.clone()),
        password: form.password.clone(),
        role: form.0.role
    };
        let user = add_user(&new_user, pool).await?;

        tracing::info!("user added : {:?}", user);
        let message = format!("Nouvel utilisateur ajouté : {}", user.user_name);

        let data = FlashData {
            kind: "success".to_owned(),
            message: message.to_owned(),
        };
        Ok(user_response(&mut cookies, data))
    }
}