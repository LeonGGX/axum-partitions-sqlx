//! src/handlers/musicians_hdl.rs

use axum::extract::{Extension, Form, Path};
use axum::response::Html;
use axum_flash::{Flash, IncomingFlashes};

use serde::{Deserialize, Serialize};

use sqlx::PgPool;
use tera::Tera;

use crate::error::AppError;
use crate::flash::person_response;

use crate::db::musicians::*;

use axum_macros::debug_handler;
use headers::HeaderMap;

use crate::globals::{get_static_vec_persons, set_static_vec_persons};
use crate::models::musician::Person;
use crate::StatusCode;
//use crate::my_askama::askama_structs::{PersonsTemplate, HtmlTemplate,};
//use askama_axum::*;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Demande {
    pub name: String,
}

//***********************************************************************************
// CRUD Operations
//

///
/// Creates a new musician in the persons table
/// and shows the list of all musicians
///
/// Returns PersonResponse or AppError
///
///
#[debug_handler]
pub async fn create_person_hdl(
    pool: Extension<PgPool>,
    form: Form<Person>,
    mut flash: Flash,
) -> Result<(StatusCode, HeaderMap), AppError> {
    let pers = form.0;

    if let Ok(person) = add_person(&pool, pers).await {
        tracing::info!("person added : {:?}", person);
        let message = format!("Musicien ajouté : {}", person.full_name);
        let level = axum_flash::Level::Success;
        Ok(person_response(&mut flash, level, message))
    } else {
        tracing::info!("error adding person");
        let message = format!("Musicien pas ajouté erreur");
        let level = axum_flash::Level::Error;
        Ok(person_response(&mut flash, level, message))
    }
}

///
/// Modifies a musician in the persons table
/// and shows the list of all musicians
///
/// Returns PersonResponse or AppError
///
///
#[debug_handler]
pub async fn update_person_hdl(
    pool: Extension<PgPool>,
    Path(id): Path<i32>,
    form: Form<Person>,
    mut flash: Flash,
) -> Result<(StatusCode, HeaderMap), AppError> {
    let updated_pers = form.0;
    let person_name = updated_pers.full_name;

    if let Ok(person) = update_person(id, person_name, &pool).await {
        let message = format!("Musicien modifié : {}", person.full_name).to_owned();
        let level = axum_flash::Level::Success;
        Ok(person_response(&mut flash, level, message))
    } else {
        let message = format!("Musicien pas modifié, erreur");
        let level = axum_flash::Level::Error;
        Ok(person_response(&mut flash, level, message))
    }
}

///
/// Deletes a musician in the persons table
/// and shows the list of all musicians
///
/// Returns PersonResponse or AppError
///
///
#[debug_handler]
pub async fn delete_person_hdl(
    pool: Extension<PgPool>,
    Path(id): Path<i32>,
    mut flash: Flash,
) -> Result<(StatusCode, HeaderMap), AppError> {
    if let Ok(deleted_name) = delete_person(id, &pool).await {
        let message = format!("Musicien effacé : {}", deleted_name).to_owned();
        let level = axum_flash::Level::Success;
        Ok(person_response(&mut flash, level, message))
    } else {
        let message = format!("Erreur Musicien pas effacé");
        let level = axum_flash::Level::Error;
        Ok(person_response(&mut flash, level, message))
    }
}

//*******************************************************************************
// Functions to show or print list of musicians
//

///
/// Shows the page with the list of musicians
///
/// Returns a HTML Page or AppError
///
#[debug_handler]
pub async fn list_persons_hdl(
    templates: Extension<Tera>,
    pool: Extension<PgPool>,
    flash: IncomingFlashes,
) -> Result<Html<String>, AppError> {
    // on va chercher le message dans IncomingFlashes pour l'afficher
    let flash = flash
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");
    tracing::info!("flash : {}", flash);

    // let persons = list_persons(&pool).await?;
    set_static_vec_persons(list_persons(&pool).await?);
    let persons = get_static_vec_persons();

    let title = "Gestion des Musiciens";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("persons", &persons);
    ctx.insert("flash", &flash);

    let body = templates
        .render("persons.html", &ctx)
        .map_err(|e| AppError::Tera(e))?;

    Ok(Html(body))
}

/*
#[debug_handler]
pub async fn askama_list_persons_hdl(
    Extension(ref pool): Extension<PgPool>,
    flash: IncomingFlashes)
//    ->  PersonsTemplate<'static> {
-> impl IntoResponse {

    let flash = flash
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");
    tracing::info!("flash : {}", flash);

    let persons = list_persons(pool).await.unwrap();
    let title = "Gestion des Musiciens".to_string();

   let template =  PersonsTemplate{
        title,
        persons,
        flash,
    };
    HtmlTemplate(template)
}
*/

///
/// Shows a printable list of Musicians
///
/// Returns a HTML Page or AppErro
///
#[debug_handler]
pub async fn print_list_persons_hdl(templates: Extension<Tera>) -> Result<Html<String>, AppError> {
    // on va chercher la liste statique
    let persons = get_static_vec_persons();
    let title = "Liste des Musiciens";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("persons", &persons);

    let body = templates
        .render("list_musicians.html", &ctx)
        .map_err(|err| AppError::Tera(err))?;

    Ok(Html(body))
}

//*************************************************************************************
// Functions to find one musician
//

///
/// find_person_by_name
///
/// returns list musicians page with musician found
///
#[debug_handler]
pub async fn find_person_by_name_hdl(
    templates: Extension<Tera>,
    pool: Extension<PgPool>,
    form: Form<Demande>,
    flash: IncomingFlashes,
) -> Result<Html<String>, AppError> {
    let demande = form.0;
    tracing::debug!("name : {:?}", demande);

    // on va chercher la liste des musiciens qui correspond à la recherche
    // si le résultat est positif ... autrement ...
    if let Ok(persons) = find_person_by_name(demande.name, &pool).await {
        // on insère le résultat de la recherche dans le Vec<Person> global
        set_static_vec_persons(persons.clone());
        // on va chercher les valeurs du Vec<Person> global (lazy_static)
        // que l'on met dans la variable que l'on affiche
        let found_persons = get_static_vec_persons();

        let title = "Personne(s) trouvée(s)";
        let flash = flash
            .into_iter()
            .map(|(level, text)| format!("{:?}: {}", level, text))
            .collect::<Vec<_>>()
            .join(", ");
        tracing::info!("flash : {}", flash);

        let mut ctx = tera::Context::new();
        ctx.insert("title", &title);
        ctx.insert("persons", &found_persons);
        ctx.insert("flash", &flash);

        let body = templates
            .render("persons.html", &ctx)
            .map_err(|err| AppError::Tera(err))?;
        return Ok(Html(body));
    } else {
        let mut ctx = tera::Context::new();
        ctx.insert("data", "personne");

        let body = templates
            .render("error/void.html.tera", &ctx)
            .map_err(|err| AppError::Tera(err))?;
        return Ok(Html(body));
    }
}
