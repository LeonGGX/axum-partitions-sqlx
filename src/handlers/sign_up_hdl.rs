//! src/handlers/sign_up_hdl.rs

use serde::{Deserialize, Serialize};

use axum::extract::{Extension, Form};
use axum::response::{Html, IntoResponse, Redirect};
use axum_database_sessions::{AxumPgPool, AxumSession};
use axum_flash::{Flash, IncomingFlashes};
use axum_macros::debug_handler;

use sqlx::PgPool;
use tera::Tera;

use crate::error::{AppError, SignupError};
use crate::flash::{error_page, login_response, signup_response};
use crate::auth::session::signup_session;


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub confirm_pwd: String,
    pub role: String,
}

/*
///
/// utilisée pour valider les champs du formulaire
///
impl TryFrom<RegisterRequest> for NewUser {
    type Error = String;

    fn try_from(value: RegisterRequest) -> Result<NewUser, Self::Error> {
        if let Ok(username) = NewUserName::parse(value.username) {
            let password = value.password;
            let role = value.role;
            Ok(Self { username, password, role })
        } else {
            Err("couldn't parse ...".to_string())
        }
    }
}
*/

///
/// Shows the signup page
/// uses Tera
/// shows flash messages
///
#[debug_handler]
pub async fn get_sign_up_hdl(
    templates: Extension<Tera>,
    flash: IncomingFlashes,
) -> Result<Html<String>, AppError> {
    let flash = flash
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");

    tracing::info!("flash : {}", flash);

    let title = "Sign Up";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("flash", &flash);

    let body = templates
        .render("sign_up.html", &ctx)
        .map_err(|e| AppError::Tera(e))?;

    Ok(Html(body))
}


#[debug_handler]
pub async fn sign_up_hdl(
    database: Extension<PgPool>,
    //Extension(random): Extension<Random>,
    session: AxumSession<AxumPgPool>,
    form: Form<RegisterRequest>,
    mut flash: Flash,
    //) -> impl IntoResponse {
) -> Result<Redirect, AppError> {
    // on vérife si les données du formulaire sont remplies
    if form.username.is_empty() {
        let message = format!("{}", SignupError::MissingUserName);
        let level = axum_flash::Level::Error;
        return Ok(signup_response(&mut flash, level, message));
    }
    if form.password.is_empty() {
        let message = format!("{}", SignupError::MissingPassword);
        let level = axum_flash::Level::Error;
        return Ok(signup_response(&mut flash, level, message));
    }
    if form.confirm_pwd.is_empty() {
        let message = format!("{}", SignupError::MissingPwConfirm);
        let level = axum_flash::Level::Error;
        return Ok(signup_response(&mut flash, level, message));
    }
    if form.role.is_empty() {
        let message = format!("{}", SignupError::MissingRole);
        let level = axum_flash::Level::Error;
        return Ok(signup_response(&mut flash, level, message));
    }

    // on vérifie si le mot de passe est bien confirmé
    if form.password != form.confirm_pwd {
        let message = format!("{}", SignupError::PasswordsDoNotMatch);
        let level = axum_flash::Level::Error;
        return Ok(signup_response(&mut flash, level, message));
    }

    match signup_session(
        &database,
        session,
        &form.username.clone(),
        &form.password.clone(),
        &form.role.clone(),
    )
    .await
    {
        Ok(uuid) => {
            let message = format!("Vous êtes enregistré avec id : {} !", uuid);
            let level = axum_flash::Level::Success;
            Ok(signup_response(&mut flash, level, message))
        }
        Err(error) => {
            let message = format!("{}", error);
            let level = axum_flash::Level::Error;
            Ok(signup_response(&mut flash, level, message))
        }
    }
}
