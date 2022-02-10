// src/handlers/partitions_hdl.rs

use axum::extract::{Extension, Form, Path};
use axum::http::{StatusCode,};
use axum::response::Html;

use tower_cookies::{Cookies,};

use tera::Tera;
use sqlx::PgPool;

use serde::{Serialize, Deserialize, };

use crate::db;

use crate::error::AppError;
use crate::flash::{
    get_flash_cookie,
    partition_response,
    PartitionResponse,
    FlashData,
};
use crate::model::{ShowPartition,};


#[derive(Deserialize, Serialize, Debug, Clone,)]
pub struct Demande {
    pub name : String,
}

pub async fn list_partitions(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    cookies: Cookies,)
    ->  Result<Html<String>, AppError> {

    let show_partitions = db::list_show_partitions(pool).await?;
    let persons = db::list_persons(pool).await?;
    let genres = db::list_genres(pool).await?;
    let title = "Gestion des Partitions";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("persons", &persons);
    ctx.insert("genres", &genres);
    ctx.insert("partitions", &show_partitions);

    if let Some(value) = get_flash_cookie::<FlashData>(&cookies) {
        ctx.insert("flash", &value);
    }

    let body = templates
        .render("partitions.html.tera", &ctx)
        .map_err(|e| AppError::Tera(e))?;

    Ok(Html(body))
}

pub async fn create_partition (
    Extension(ref pool): Extension<PgPool>,
    form: Form<ShowPartition>,
    mut cookies: Cookies,
)-> Result<PartitionResponse, (StatusCode, &'static str)> {

    let show_partition = form.0;

    let person_name = show_partition.full_name.clone();
    let genre_name = show_partition.name.clone();
    let title = show_partition.title.clone();

    let new_partition =
        db::add_partition(title,person_name, genre_name, pool).await;

    match new_partition {
        Ok(new_partition) => {
            tracing::info!("nouvelle partition : {:?}", new_partition);

            let message = format!("Partition ajoutée : {}", new_partition.title);

            let data = FlashData {
                kind: "success".to_owned(),
                message: message.to_owned(),
            };
            Ok(partition_response(&mut cookies, data))
        }
        Err(_) => {
            let message = format!("Erreur : Partition pas ajoutée !");

            let data = FlashData {
                kind: "error".to_owned(),
                message: message.to_owned(),
            };
            Ok(partition_response(&mut cookies, data))
        }
    }
}


pub async fn update_partition(
    Extension(ref pool): Extension<PgPool>,
    Path(id): Path<i32>,
    form: Form<ShowPartition>,
    mut cookies: Cookies,
)->  Result<PartitionResponse, AppError> {

    let show_partition = form.0;

    let person = db::find_person_by_name(show_partition.full_name, pool).await?;
    let person_id = person.id.unwrap();

    let genre = db::find_genre_by_name(show_partition.name, pool).await?;
    let genre_id = genre.id.unwrap();

    let title = show_partition.title;

    let partition_changed = db::update_partition(id, title, person_id, genre_id, pool).await?;
    let data = FlashData {
        kind: "success".to_owned(),
        message: format!("Partition successfully updated : {:?}", partition_changed).to_owned(),
    };

    Ok(partition_response(&mut cookies, data))

}


pub async fn delete_partition(
    Extension(ref pool): Extension<PgPool>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
) -> Result<PartitionResponse, AppError>  {

    let partition_title = db::delete_partition(id, pool).await?;

    let data = FlashData {
        kind: "success".to_owned(),
        message: format!("Partition succcessfully deleted : {}", partition_title).to_owned(),
    };

    Ok(partition_response(&mut cookies, data))
}


pub async fn print_list_partitions(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    _cookies: Cookies,
)->  Result<Html<String>, AppError> {

    let show_partitions = db::list_show_partitions(pool).await.unwrap();
    let title = "liste des partitions";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("partitions", &show_partitions);

    let body = templates
        .render("list_partitions.html.tera", &ctx)
        .map_err(|err| AppError::Tera(err))?;

    Ok(Html(body))
}


pub async fn find_partition_title(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    form: Form<Demande>,
    _cookies: Cookies,
)->  Result<Html<String>, AppError> {

    let demande = form.0;
    let name = demande.name;

    if let Ok(partition) = db::find_partition_by_title(name, pool).await {

        let title = "Partition(s) trouvée(s)";

        let show_partition = db::show_one_partition(partition, pool).await.unwrap();

        let mut partitions: Vec<ShowPartition> = Vec::new();
        partitions.push(show_partition);

        let persons = db::list_persons(pool).await?;
        let genres = db::list_genres(pool).await?;

        let mut ctx = tera::Context::new();
        ctx.insert("title", &title);
        ctx.insert("partitions", &partitions);
        ctx.insert("genres", &genres);
        ctx.insert("persons", &persons);

        let body = templates
            .render("partitions.html.tera", &ctx)
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

pub async fn find_partition_genre(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    form: Form<Demande>,
    _cookies: Cookies,) -> Result<Html<String>, AppError> {

    let demande = form.0;
    let name = demande.name;

    let title = "Partition(s) trouvée(s)";

    let partitions = db::find_partition_by_genre(name, pool).await?;
    let mut show_partitions: Vec<ShowPartition> = Vec::new();
    for partition in partitions {
        let show_part = db::show_one_partition(partition, pool).await?;
        show_partitions.push(show_part);
    }
    let persons = db::list_persons(pool).await?;
    let genres = db::list_genres(pool).await?;

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("partitions", &show_partitions);
    ctx.insert("genres", &genres);
    ctx.insert("persons", &persons);

    let body = templates
        .render("partitions.html.tera", &ctx)
        .map_err(|err| AppError::Tera(err))?;
    Ok(Html(body))

}

pub async fn find_partition_author(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    form: Form<Demande>,
    _cookies: Cookies,) -> Result<Html<String>, AppError>{

    let demande = form.0;
    let name = demande.name;

    let title = "Partition(s) trouvée(s)";

    let partitions = db::find_partition_by_author(name, pool).await?;
    let mut show_partitions: Vec<ShowPartition> = Vec::new();
    for partition in partitions {
        let show_part = db::show_one_partition(partition, pool).await?;
        show_partitions.push(show_part);
    }
    let persons = db::list_persons(pool).await?;
    let genres = db::list_genres(pool).await?;

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("partitions", &show_partitions);
    ctx.insert("genres", &genres);
    ctx.insert("persons", &persons);

    let body = templates
        .render("partitions.html.tera", &ctx)
        .map_err(|err| AppError::Tera(err))?;

    Ok(Html(body))
}


