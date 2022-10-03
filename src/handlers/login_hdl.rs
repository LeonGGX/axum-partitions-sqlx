//! src/handlers/login_hdl.rs

use axum::{extract::{Extension, Form, },
           response::{Html, IntoResponse, Redirect, },
};
use axum_database_sessions::{AxumPgPool, AxumSession};
use axum_flash::{Flash, IncomingFlashes};
use axum_macros::debug_handler;

use sqlx::PgPool;

use tera::Tera;

use headers::HeaderMap;

use crate::{AppError, StatusCode};
use crate::flash::{error_page, login_response};
use crate::auth::jwt::LoginPayload;
use crate::auth::session::login_session;
use crate::error::LoginError;

///
/// affiche la page de login
/// affiche les messages flash
///
#[debug_handler]
pub async fn login_form_hdl(
    Extension(templates): Extension<Tera>,
    flash: IncomingFlashes,
) -> Result<Html<String>, AppError> {
    let flash = flash
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");
    tracing::info!("flash : {}", flash);

    let title = "Login - S'identifier";

    let mut ctx = tera::Context::new();
    ctx.insert("flash", &flash);
    ctx.insert("title", &title);

    let body = templates
        .render("login.html", &ctx)
        .map_err(|e| AppError::Tera(e))?;

    Ok(Html(body))
}

#[debug_handler]
pub async fn login_hdl(
    database: Extension<PgPool>,
    session: AxumSession<AxumPgPool>,
    form: Form<LoginPayload>,
    mut flash: Flash,
    //) -> impl IntoResponse {
    //)-> Result<(StatusCode, HeaderMap), AppError> {
) -> Result<Redirect, AppError> {
    // on vérifie si les données du formulaire sont présentes
    if form.username.is_empty() {
        let message = format!("{}", LoginError::MissingUserName);
        let level = axum_flash::Level::Error;
        return Ok(login_response(&mut flash, level, message));
    }
    if form.password.is_empty() {
        let message = format!("{}", LoginError::MissingPassword);
        let level = axum_flash::Level::Error;
        return Ok(login_response(&mut flash, level, message));
    }

    match login_session(
        &database,
        session,
        form.username.clone(),
        form.password.clone(),
    )
    .await
    {
        Ok(session) => {
            tracing::info!("session : {:?}", session);
            let message = "Vous êtes loggé !".to_string();
            let level = axum_flash::Level::Success;
            Ok(login_response(&mut flash, level, message))
        }
        Err(err) => {
            let message = format!("{}", &err);
            let level = axum_flash::Level::Error;
            Ok(login_response(&mut flash, level, message))
        }
    }
}
