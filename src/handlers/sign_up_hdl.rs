//! src/handlers/sign_up_hdl.rs

use serde::{Deserialize, Serialize};

use axum::extract::{Extension, Form};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, /*IntoResponse,*/};
use axum_flash::{Flash, IncomingFlashes};

use sqlx::PgPool;
use tera::Tera;

use unicode_segmentation::UnicodeSegmentation;

use crate::error::AppError;
use crate::flash::{signup_response,};
use crate::models::user::{NewUser, NewUserName};
use crate::{auth, db,};

use axum_macros::debug_handler;

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
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
    flash: IncomingFlashes) -> Result<Html<String>, AppError > {

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

///
/// Handles signup data from signup page
/// shows flash messages
///
#[debug_handler]
pub async fn sign_up_hdl(
    pool: Extension<PgPool>,
    form: Form<RegisterRequest>,
    mut flash: Flash,
) -> Result<(StatusCode, HeaderMap), AppError> {

    // on vérife si les données du formulaire sont remplies
    if form.username.is_empty() {
        let message = "Il faut entrer un nom d'utilisateur".to_string();
        return Ok(signup_response(&mut flash, message));
    }
    if form.password.is_empty(){
        let message = "Il faut entrer un mot de passe".to_string();
        return Ok(signup_response(&mut flash, message));
    }
    if form.role.is_empty(){
        let message = "Il faut entrer un rôle".to_string();
        return Ok(signup_response(&mut flash, message));
    }

    // parse vérifie si le nom d'utilisateur est correct
    // on vérifie ensuite si le futur utilisateur existe dans la DB afin d'éviter les doublons
    // on crée un nouvel utilisateur NewUser avec un mot de passe crypté
    // on ajoute le nouvel utilisateur à la DB
    // on renvoie vers la page signup avec le message de création du nouvel utilisateur
    if let Ok(name) = parse(&form.username) {
        if let Ok(user) = db::users::find_user_by_name(name, &pool).await {
            let message = format!("L'utilisateur {:?} existe déjà !", user.name);
            Ok(signup_response(&mut flash, message))
            //return Err(AppError::UserExists);
        }
        else {
            let name = NewUserName::parse(form.username.clone()).unwrap();
            // fonction de hash ici
            let password = auth::hash_password(form.password.clone()).await.unwrap();
            // *********************
            let new_user = NewUser{
                name,
                password,
                role: form.role.clone(),
            };
            let added_user = db::users::add_user(&new_user, &pool).await.unwrap();
            tracing::info!("user added : {:?}", added_user);
            let message = format!("Nouvel utilisateur ajouté : {}", added_user.name);
            return Ok(signup_response(&mut flash, message));
        }

    } else {
        //return Err(AppError::ValidationError);
        let message = format!("Erreur de validation");
        return Ok(signup_response(&mut flash, message));
    }
}

pub fn parse(s: &String) -> Result<String, AppError> {
    // `.trim()` returns a view over the input `s` without trailing
    // whitespace-like characters.
    // `.is_empty` checks if the view contains any character.
    let is_empty_or_whitespace = s.trim().is_empty();

    // A grapheme is defined by the Unicode standard as a "user-perceived"
    // character: `å` is a single grapheme, but it is composed of two characters
    // (`a` and `̊`).
    //
    // `graphemes` returns an iterator over the graphemes in the input `s`.
    // `true` specifies that we want to use the extended grapheme definition set,
    // the recommended one.
    let is_too_long = s.graphemes(true).count() > 256;

    // Iterate over all characters in the input `s` to check if any of them matches
    // one of the characters in the forbidden array.
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

    if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
        return Err(AppError::ValidationError);
        //panic!("{} is not a valid new user name.", s)
    } else {
        return Ok(s.to_string());
    }
}


