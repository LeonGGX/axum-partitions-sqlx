//! src/handlers/genres_hdl.rs

use axum::extract::{Extension, Form, Path};
use axum::response::Html;

use tower_cookies::{Cookies,};

use tera::Tera;
use sqlx::PgPool;
use serde::{Serialize, Deserialize,};

use crate::db::genres::*;

use crate::error::AppError;
use crate::flash::{
    get_flash_cookie,
    genre_response,
    GenreResponse,
    FlashData
};
use crate::model::{Genre,};

#[derive(Deserialize, Serialize, Debug, Clone,)]
pub struct Demande {
    pub name : String,
}


//***********************************************************************************
// CRUD Operations
//

///
/// Create a new genre in the genres table
/// and shows the list of all genres
///
/// Returns GenreResponse or AppError
///

pub async fn create_genre_hdl(
    Extension(ref pool): Extension<PgPool>,
    form: Form<Genre>,
    mut cookies: Cookies,
)-> Result<GenreResponse, AppError> {

    let genre = form.0;
    let new_genre = add_genre(pool, genre).await?;
    let message = format!("Genre ajouté : {}", new_genre.name);

    let data = FlashData {
        kind: "success".to_owned(),
        message: message.to_owned(),
    };

    Ok(genre_response(&mut cookies, data))
}

pub async fn update_genre_hdl(
    Extension(ref pool): Extension<PgPool>,
    Path(id): Path<i32>,
    form: Form<Demande>,
    mut cookies: Cookies,
)->  Result<GenreResponse, AppError> {

    let updated_genre = form.0;
    let genre_name = updated_genre.name;

    let genre = update_genre(id, genre_name, pool).await?;

    let data = FlashData {
        kind: "success".to_owned(),
        message: format!("Genre successfully updated : {:?}", genre).to_owned(),
    };

    Ok(genre_response(&mut cookies, data))
}

pub async fn delete_genre_hdl(
    Extension(ref pool): Extension<PgPool>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
) -> Result<GenreResponse, AppError> {

    let genre_nom = delete_genre(id, pool).await?;

    let data = FlashData {
        kind: "success".to_owned(),
        message: format!("Genre succcessfully deleted : {}", genre_nom).to_owned(),
    };

    Ok(genre_response(&mut cookies, data))
}

//*******************************************************************************
// Functions to show or print list of genres
//

///
/// Shows the page with the list of genres
///
/// Returns a HTML Page or AppError
///
pub async fn list_genres_hdl(Extension(ref templates): Extension<Tera>,
                         Extension(ref pool): Extension<PgPool>,
                         cookies: Cookies,)->  Result<Html<String>, AppError> {

    let genres = list_genres(pool).await?;
    let title = "Gestion des Genres";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("genres", &genres);

    if let Some(value) = get_flash_cookie::<FlashData>(&cookies) {
        ctx.insert("flash", &value);
    }

    let body = templates
        .render("genres.html.tera", &ctx)
        .map_err(|err| AppError::Tera(err))?;

    Ok(Html(body))
}

///
/// Shows a printable list of Genres
///
/// Returns a HTML Page or AppError
///
pub async fn print_list_genres_hdl(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    _cookies: Cookies,
)->  Result<Html<String>, AppError> {

    let genres = list_genres(pool).await?;

    let title = "Liste des Genres";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("genres", &genres);

    let body = templates
        .render("list_genres.html.tera", &ctx)
        .map_err(|err| AppError::Tera(err))?;

    Ok(Html(body))
}

//****************************************************************************************
// Functions to find genres by different criteria
//

///
/// find_genre_by_name
///
/// returns list genre page with genre found
///
pub async fn find_genre_by_name_hdl(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    form: Form<Demande>,
    _cookies: Cookies,
)->  Result<Html<String>, AppError> {

    let demande = form.0;
    tracing::debug!("name : {:?}", demande);

    let name = demande.name;

    let genres = find_genre_by_name(name, pool).await?;

    let title = "Genre(s) trouvé(s)";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("genres", &genres);

    let body = templates
        .render("genres.html.tera", &ctx)
        .map_err(|err| AppError::Tera(err))?;

    Ok(Html(body))
}
