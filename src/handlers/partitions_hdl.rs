//! src/handlers/partitions_hdl.rs

use axum::extract::{Extension, Form, Path};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Html;
use axum_flash::{Flash, IncomingFlashes};

//use tower_cookies::{Cookies,};

use tera::Tera;
use sqlx::PgPool;

use serde::{Serialize, Deserialize, };

use crate::db::{
    genres::*,
    musicians::*,
    partitions::*,
    partitions::update_partition,
};

use crate::error::AppError;
use crate::flash::partition_response;
use crate::globals::{get_static_vec_partitions, set_static_vec_partitions};
use crate::models::genre::Genre;
use crate::models::musician::Person;
use crate::models::partition::ShowPartition;



#[derive(Deserialize, Serialize, Debug, Clone,)]
pub struct Demande {
    pub name : String,
}

//***********************************************************************************
// CRUD Operations
//

///
/// Create a new partition in the partitions table
/// and shows the list of all partitions
///
/// Returns PartitionResponse or AppError
///
pub async fn create_partition_hdl (
    //Extension(ref pool): Extension<PgPool>,
    pool: Extension<PgPool>,
    form: Form<ShowPartition>,
    mut flash: Flash,
)-> Result<(StatusCode, HeaderMap), (StatusCode, &'static str)> {

    let show_partition = form.0;

    let person_name = show_partition.full_name.clone();
    let genre_name = show_partition.name.clone();
    let title = show_partition.title.clone();

    let new_partition =
        add_partition(title,person_name, genre_name, &pool).await;

    match new_partition {
        Ok(new_partition) => {
            tracing::info!("nouvelle partition : {:?}", new_partition);

            let message = format!("Partition ajoutée : {}", new_partition.title);
            Ok(partition_response(&mut flash, message))
        }
        Err(_) => {
            let message = format!("Erreur : Partition pas ajoutée !");
            Ok(partition_response(&mut flash, message))
        }
    }
}


pub async fn update_partition_hdl(
    Extension(ref pool): Extension<PgPool>,
    Path(id): Path<i32>,
    form: Form<ShowPartition>,
    mut flash: Flash,
)->  Result<(StatusCode, HeaderMap), AppError> {

    let show_partition = form.0;

    let person = find_person_by_name(show_partition.full_name, pool).await?;
    let person_id = person[0].id.unwrap();

    let genre = find_genre_by_name(show_partition.name, pool).await?;
    let genre_id = genre[0].id.unwrap();

    let title = show_partition.title;

    let partition_changed = update_partition(id, title, person_id, genre_id, pool).await?;

    let message = format!("Partition successfully updated : {:?}", partition_changed).to_owned();
    Ok(partition_response(&mut flash, message))
}


pub async fn delete_partition_hdl(
    Extension(ref pool): Extension<PgPool>,
    Path(id): Path<i32>,
    mut flash: Flash,
) -> Result<(StatusCode, HeaderMap), AppError>  {

    let partition_title = delete_partition(id, pool).await?;
    let message = format!("Partition succcessfully deleted : {}", partition_title).to_owned();

    Ok(partition_response(&mut flash, message))
}

//*******************************************************************************
// Functions to show or print list of partitions
//

///
/// Shows the page with the list of partitions via ShowPartition
///
/// Returns a HTML Page or AppError
///
pub async fn list_partitions_hdl(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    flash: IncomingFlashes,)
    ->  Result<Html<String>, AppError> {

    let flash = flash
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");
    tracing::info!("flash : {}", flash);

    //let show_partitions = list_show_partitions(pool).await?;
    set_static_vec_partitions(list_show_partitions(pool).await?);
    let show_partitions = get_static_vec_partitions();

    let persons = list_persons(pool).await?;
    let genres = list_genres(pool).await?;
    let title = "Gestion des Partitions";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("persons", &persons);
    ctx.insert("genres", &genres);
    ctx.insert("partitions", &show_partitions);
    ctx.insert("flash", &flash);

    let body = templates
        .render("partitions.html", &ctx)
        .map_err(|e| AppError::Tera(e))?;

    Ok(Html(body))
}

///
/// Shows a printable list of all partitions in the db
/// under the form of ShowPartitions
///
/// Returns a HTML Page or AppError
///
pub async fn print_list_partitions_hdl(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
)->  Result<Html<String>, AppError> {

    //let show_partitions = list_show_partitions(pool).await.unwrap();
    let show_partitions = get_static_vec_partitions();

    let title = "liste des partitions";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("partitions", &show_partitions);

    let body = templates
        .render("list_partitions.html", &ctx)
        .map_err(|err| AppError::Tera(err))?;

    Ok(Html(body))
}

//*************************************************************************************
// Functions to find one or several partitions based on different criteria
//

///
/// find_partition_by_title
///
/// returns list musicians page with partition(s) found by title
///
pub async fn find_partition_title_hdl(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    form: Form<Demande>,
    flash: IncomingFlashes,
)->  Result<Html<String>, AppError> {

    let flash = flash
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");
    tracing::info!("flash : {}", flash);

    let demande = form.0;
    let name = demande.name;

    if let Ok(partitions) = find_partition_by_title(name, pool).await {

        let title = "Partition(s) trouvée(s)";
/*
        let mut show_partitions: Vec<ShowPartition> = Vec::new();
        for partition in partitions {
            let one_show_partition = show_one_partition(partition, pool).await.unwrap();
            show_partitions.push(one_show_partition);
        }
*/
        let sh_partitions = vec_showpartitions_from_vec_partitions(partitions, pool).await;
        set_static_vec_partitions(sh_partitions);
        let show_partitions = get_static_vec_partitions();

        let persons = list_persons(pool).await?;
        let genres = list_genres(pool).await?;

        let mut ctx = tera::Context::new();
        ctx.insert("title", &title);
        ctx.insert("partitions", &show_partitions);
        ctx.insert("genres", &genres);
        ctx.insert("persons", &persons);

        let body = templates
            .render("partitions.html", &ctx)
            .map_err(|err| AppError::Tera(err))?;
        Ok(Html(body))
    }
    else {
        let mut ctx = tera::Context::new();
        ctx.insert("data", "partition");

        let body = templates
            .render("error/void.html.tera", &ctx)
            .map_err(|err| AppError::Tera(err))?;
        Ok(Html(body))
    }
}

pub async fn find_partition_genre_hdl(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    form: Form<Genre>,
   ) -> Result<Html<String>, AppError> {

    let genre = form.0;
    let name = genre.name;

    let title = "Partition(s) trouvée(s)";

    let partitions = find_partition_by_genre(name, pool).await?;
    let mut show_partitions: Vec<ShowPartition> = Vec::new();
    for partition in partitions {
        let show_part = show_one_partition(partition, pool).await?;
        show_partitions.push(show_part);
    }
    let persons = list_persons(pool).await?;
    let genres = list_genres(pool).await?;

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("partitions", &show_partitions);
    ctx.insert("genres", &genres);
    ctx.insert("persons", &persons);

    let body = templates
        .render("partitions.html", &ctx)
        .map_err(|err| AppError::Tera(err))?;
    Ok(Html(body))

}

pub async fn find_partition_author_hdl(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    form: Form<Person>,
   ) -> Result<Html<String>, AppError>{

    let person = form.0;
    let name = person.full_name;

    let title = "Partition(s) trouvée(s)";

    let partitions = find_partition_by_author(name, pool).await?;
    let mut show_partitions: Vec<ShowPartition> = Vec::new();
    for partition in partitions {
        let show_part = show_one_partition(partition, pool).await?;
        show_partitions.push(show_part);
    }
    set_static_vec_partitions(show_partitions);
    let show_partitions = get_static_vec_partitions();

    let persons = list_persons(pool).await?;
    let genres = list_genres(pool).await?;

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("partitions", &show_partitions);
    ctx.insert("genres", &genres);
    ctx.insert("persons", &persons);

    let body = templates
        .render("partitions.html", &ctx)
        .map_err(|err| AppError::Tera(err))?;

    Ok(Html(body))
}


