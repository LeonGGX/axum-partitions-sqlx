//! src/handlers/genres_hdl.rs

use axum::extract::{Extension, Form, Path};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Html;
use axum_flash::{Flash, IncomingFlashes};

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tera::Tera;

use crate::db::genres::*;

use crate::error::AppError;
use crate::flash::genre_response;
use crate::globals::{get_static_vec_genres, set_static_vec_genres};
use crate::models::genre::Genre;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Demande {
    pub name: String,
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
    mut flash: Flash,
    //mut cookies: Cookies,
) -> Result<(StatusCode, HeaderMap), AppError> {
    let genre = form.0;
    let new_genre = add_genre(pool, genre).await?;
    let message = format!("Genre ajouté : {}", new_genre.name);

    Ok(genre_response(&mut flash, message))
}

pub async fn update_genre_hdl(
    Extension(ref pool): Extension<PgPool>,
    Path(id): Path<i32>,
    form: Form<Demande>,
    mut flash: Flash,
    //mut cookies: Cookies,
) -> Result<(StatusCode, HeaderMap), AppError> {
    let updated_genre = form.0;
    let genre_name = updated_genre.name;
    let genre = update_genre(id, genre_name, pool).await?;
    let message = format!("Genre modifié avec succès : {:?}", genre.name).to_owned();
    Ok(genre_response(&mut flash, message))
}

pub async fn delete_genre_hdl(
    Extension(ref pool): Extension<PgPool>,
    Path(id): Path<i32>,
    mut flash: Flash,
) -> Result<(StatusCode, HeaderMap), AppError> {
    let genre_nom = delete_genre(id, pool).await?;
    let message = format!("Genre effacé : {}", genre_nom).to_owned();

    Ok(genre_response(&mut flash, message))
}

//*******************************************************************************
// Functions to show or print list of genres
//

///
/// Shows the page with the list of genres
///
/// Returns a HTML Page or AppError
///
pub async fn list_genres_hdl(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    flash: IncomingFlashes,
) -> Result<Html<String>, AppError> {
    let flash = flash
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");
    tracing::info!("flash : {}", flash);

    //let genres = list_genres(pool).await?;
    set_static_vec_genres(list_genres(pool).await?);
    let genres = get_static_vec_genres();

    let title = "Gestion des Genres";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("genres", &genres);
    ctx.insert("flash", &flash);

    let body = templates
        .render("genres.html", &ctx)
        .map_err(|err| AppError::Tera(err))?;

    Ok(Html(body))
}

///
/// Shows a printable list of Genres
///
/// Returns a HTML Page or AppError
///
pub async fn print_list_genres_hdl(
    //Extension(ref templates): Extension<Tera>,
    //templates: Extension<Arc<Tera>>,
    templates: Extension<Tera>,
    //Extension(ref pool): Extension<PgPool>,
) -> Result<Html<String>, AppError> {
    //let genres = list_genres(pool).await?;
    let genres = get_static_vec_genres();

    let title = "Liste des Genres";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("genres", &genres);

    let body = templates
        .render("list_genres.html", &ctx)
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
) -> Result<Html<String>, AppError> {
    let demande = form.0;
    tracing::debug!("name : {:?}", demande);

    let name = demande.name;
    //let genres = find_genre_by_name(name, pool).await?;
    set_static_vec_genres(find_genre_by_name(name, pool).await?);
    let genres = get_static_vec_genres();

    let title = "Genre(s) trouvé(s)";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("genres", &genres);

    let body = templates
        .render("genres.html", &ctx)
        .map_err(|err| AppError::Tera(err))?;

    Ok(Html(body))
}
