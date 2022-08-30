//! src/handlers/login_hdl.rs

use axum::extract::{Extension, Form};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html,};

use axum_flash::{Flash, IncomingFlashes};

use jsonwebtoken::{encode, Header};
use sqlx::PgPool;
use once_cell::sync::Lazy;
use serde::Deserialize;
use tera::Tera;

use crate::{AppError, auth, db};
use crate::auth::{Claims, Keys,};
use crate::flash::login_response;

use axum_macros::debug_handler;

static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

///
/// affiche la page de login
/// affiche les messages flash
///
#[debug_handler]
pub async fn login_form_hdl(
    templates: Extension<Tera>,
    flash: IncomingFlashes,) -> Result<Html<String>, AppError> {

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

///
/// Traite les données de login et retourne un Token
/// Affiche les messages flash
///
#[debug_handler]
pub async fn login_hdl(
    pool: Extension<PgPool>,
    form: Form<LoginPayload>,
    mut flash: Flash,) -> Result<(StatusCode, HeaderMap), AppError> {

    // on vérifie si les données du formulaire sont présentes
    if form.username.is_empty() {
        let message = "Il faut entrer un nom d'utilisateur".to_string();
        return Ok(login_response(&mut flash, message));
    }
    if form.password.is_empty(){
        let message = "Il faut entrer un mot de passe".to_string();
        return Ok(login_response(&mut flash, message));
    }

    // on va voir si l'utilisateur est repris dans la DB
    // si oui, on va chercher son mot de passe
    let user_in_db =
        db::users::find_user_by_name(
            form.username.clone(),
            &pool).await.unwrap();
    let user_in_db_pwd = user_in_db.password_hash;

    // on vérifie si le mot de passe du formulaire correspond à celui de la DB
    // s'il correspond on crée un token et on le renvoie sous forme de message flash
    match auth::verify_password(
        form.password.clone(), user_in_db_pwd).await {
        Ok(_) => {
            let claims = Claims {
            sub: user_in_db.id,
            username: user_in_db.name,
            exp: 100000,
        };
            // Create the authorization token
            let token = encode(&Header::default(), &claims, &KEYS.encoding)?;
            //.map_err(|e| AppError::JWTTokenCreationError(e));

            // Send the authorized token
            let message = format!("Token créé : {}", token);
            return Ok(login_response(&mut flash, message));
        }
        Err(err) => {
            let message = format!("Erreur : {:?}", err);
            return Ok(login_response(&mut flash, message));
        }
    }
}



