//! src/handlers/musicians_hdl.rs

use axum::extract::{Extension, Form, Path};
use axum::response::Html;

use serde::{Deserialize, Serialize,};

use tower_cookies::{Cookies,};

use tera::Tera;
use sqlx::PgPool;

use crate::db::musicians;

use crate::error::AppError;
use crate::flash::{
    get_flash_cookie,
    person_response,
    PersonResponse,
    FlashData,
};
use crate::model::{Person, };
use crate::db::musicians::*;


#[derive(Deserialize, Serialize, Debug, Clone,)]
pub struct Demande {
    pub name : String,
}

//***********************************************************************************
// CRUD Operations
//

///
/// Create a new musician in the persons table
/// and shows the list of all musicians
///
/// Returns PersonResponse or AppError
///
pub async fn create_person_hdl(
    Extension(ref pool): Extension<PgPool>,
    form: Form<Person>,
    mut cookies: Cookies,
)-> Result<PersonResponse, AppError> {

    let pers = form.0;
    let person = add_person(pool, pers).await?;

    tracing::info!("person added : {:?}", person);
    let message = format!("Personne ajoutée : {}", person.full_name);

    let data = FlashData {
        kind: "success".to_owned(),
        message: message.to_owned(),
    };

    Ok(person_response(&mut cookies, data))
}

pub async fn update_person_hdl(
    Extension(ref pool): Extension<PgPool>,
    Path(id): Path<i32>,
    form: Form<Person>,
    mut cookies: Cookies,
)->  Result<PersonResponse, AppError> {

    let updated_pers = form.0;
    let person_name = updated_pers.full_name;

    let person = update_person(id,person_name, pool).await?;
    let data = FlashData {
        kind: "success".to_owned(),
        message: format!("Person successfully updated : {:?}", person).to_owned(),
    };

    Ok(person_response(&mut cookies, data))
}

pub async fn delete_person_hdl(
    Extension(ref pool): Extension<PgPool>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
) -> Result<PersonResponse, AppError> {

    let del = delete_person(id, pool).await?;

    let data = FlashData {
        kind: "success".to_owned(),
        message: format!("Person successfully deleted: {}", del).to_owned(),
    };

    Ok(person_response(&mut cookies, data))
}

//*******************************************************************************
// Functions to show or print list of musicians
//

///
/// Shows the page with the list of musicians
///
/// Returns a HTML Page or AppError
///
pub async fn list_persons_hdl(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    cookies: Cookies,)
    ->  Result<Html<String>, AppError> {

    let persons = list_persons(pool).await?;
    let title = "Gestion des Musiciens";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("persons", &persons);

    if let Some(value) = get_flash_cookie::<FlashData>(&cookies) {
        ctx.insert("flash", &value);
    }

    let body = templates
        .render("persons.html.tera", &ctx)
        .map_err(|e| AppError::Tera(e))?;

    Ok(Html(body))
}

///
/// Shows a printable list of Musicians
///
/// Returns a HTML Page or AppErro
///
pub async fn print_list_persons_hdl(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    _cookies: Cookies,
)->  Result<Html<String>, AppError> {

    let persons = list_persons(pool).await?;
    let title = "Liste des Personnes";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("persons", &persons);

    let body = templates
        .render("list_users.html.tera", &ctx)
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
pub async fn find_person_by_name_hdl(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    form: Form<Demande>,
    _cookies: Cookies,
)->  Result<Html<String>, AppError> {

    let demande = form.0;
    tracing::debug!("name : {:?}", demande);

    //let name = demande.name;

    let persons = find_person_by_name(demande.name, pool).await;
    match persons {
        Ok(persons) => {
            let title = "Personne(s) trouvée(s)";

            let mut ctx = tera::Context::new();
            ctx.insert("title", &title);
            ctx.insert("persons", &persons);

            let body = templates
                .render("persons.html.tera", &ctx)
                .map_err(|err| AppError::Tera(err))?;
            Ok(Html(body))
        }
        Err(_) => {
            let mut ctx = tera::Context::new();
            ctx.insert("data", "personne");

            let body = templates
                .render("error/void.html.tera", &ctx)
                .map_err(|err| AppError::Tera(err))?;
            Ok(Html(body))
        }
    }
}