// src/main.rs

mod db;
mod flash;
mod persons;
mod genre;
mod partition;
mod model;
mod error;

use axum::{
    extract::{Form, Extension, Path,},
    http::{StatusCode, Uri,},
    response::{Html},
    routing::{get, post, get_service},
    Router,
    AddExtensionLayer,
};

use axum_debug::debug_handler;

use serde::{Deserialize, Serialize};
use std::{env, net::SocketAddr};
use std::str::FromStr;
use axum::extract::Query;

use tera::Tera;

use tokio::signal;

use tower::ServiceBuilder;
use tower_cookies::{CookieManagerLayer, Cookies,};
use tower_http::services::ServeDir;

use persons::Entity as SeaOrmPerson;
use genre::Entity as SeaOrmGenre;
use sea_orm::{prelude::*, Database, Set, Order, QueryOrder};
use sqlx::PgPool;

use crate::error::AppError;
use crate::flash::{
    get_flash_cookie,
    person_response,
    PersonResponse,
    genre_response,
    GenreResponse,
    partition_response,
    PartitionResponse
};
use crate::model::{Person, Genre, ShowPartition, Partition};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "axum_jwt=debug")
    }
    tracing_subscriber::fmt::init();

    env::set_var( "JWT_SECRET", "secret");

    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{}:{}", host, port);

    // ici utilisation de SeaOrm
    let conn = Database::connect(db_url.clone())
        .await
        .expect("Database connection failed");

    // ici utilisation de sqlx
    let pool = db::create_pg_pool(&db_url).await?;

    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"))
        .expect("Tera initialization failed");

    let app = Router::new()
        .fallback(get(handler_404))
        .route("/", get(root))
        .route("/about", get(about))

        .route("/persons", get(list_persons))
        .route("/persons/add", post(create_person))
        .route("/persons/:id", post(update_person))
        .route("/persons/delete/:id", post(delete_person))
        .route("/persons/print", get(print_list_persons))
        .route("/persons/find", post(find_person_by_name))

        .route("/genres", get(list_genres))
        .route("/genres/add", post(create_genre))
        .route("/genres/:id", post(update_genre))
        .route("/genres/delete/:id", post(delete_genre))
        .route("/genres/print", get(print_list_genres))
        .route("/genres/find", post(find_genre_by_name))

        .route("/partitions", get(list_partitions))
        .route("/partitions/add", post(create_partition))
        //.route("/partitions/:id", post(update_partition))
        .route("/partitions/delete/:id", post(delete_partition))
        .route("/partitions/print", get(print_list_partitions))
        .route("/partitions/find/title", post(find_partition_title))
        .route("/partitions/find/genre", post(find_partition_genre))
        .route("/partitions/find/author", post(find_partition_author))

        .nest(
            "/static",
                get_service(ServeDir::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static"
                )))
                    .handle_error(|error: std::io::Error| async move {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {}", error),
                        )
                    }),
        )
        .layer(
            ServiceBuilder::new()
                .layer(CookieManagerLayer::new())
                .layer(AddExtensionLayer::new(conn))
                .layer(AddExtensionLayer::new(pool))
                .layer(AddExtensionLayer::new(templates)));

    let addr = SocketAddr::from_str(&server_url).unwrap();
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct FlashData {
    kind: String,
    message: String,
}
// Il faut une fonction root qui ramène à la racine
// sinon problème. Sauf si on utilise Redirect
//
async fn root(Extension(ref templates): Extension<Tera>,) -> Result<Html<String>, AppError> {

    let title = "Start";
    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    let body = templates
        .render("start.html.tera", &ctx)
        .map_err(|e| AppError::Tera(e))?;

    Ok(Html(body))
}

async fn about(
    Extension(ref templates): Extension<Tera>,
) -> Result<Html<String>, AppError> {
    let title = "A propos de ...";
    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    let body = templates
        .render("about.html.tera", &ctx)
        .map_err(|e| AppError::Tera(e))?;

    Ok(Html(body))
}


async fn handler_404(
    Extension(ref templates): Extension<Tera>,
    uri: Uri,
) -> Result<Html<String>, (StatusCode, &'static str)> {

    let origin = uri.path();
    //println!("uri : {}", origin);
    let mut ctx = tera::Context::new();
    ctx.insert("uri", origin);
    let body = templates
        .render("error/404.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error in 404.html.tera"))?;

    Ok(Html(body))
}

//******************************************************************************************
// handlers to show lists of rows of DB tables
//

async fn list_persons(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    cookies: Cookies,)
    ->  Result<Html<String>, AppError> {

    let persons = db::list_persons(pool).await?;
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

async fn list_genres(Extension(ref templates): Extension<Tera>,
                     Extension(ref pool): Extension<PgPool>,
                     cookies: Cookies,)->  Result<Html<String>, AppError> {

    let genres = db::list_genres(pool).await?;
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

async fn list_partitions(
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

//***************************************************************************************
// handlers to create rows in DB
//

async fn create_person(
    Extension(ref pool): Extension<PgPool>,
    form: Form<Person>,
    mut cookies: Cookies,
)-> Result<PersonResponse, AppError> {

    let pers = form.0;
    let person = db::add_person(pool, pers).await?;

    tracing::info!("person added : {:?}", person);
    let message = format!("Personne ajoutée : {}", person.full_name);

    let data = FlashData {
        kind: "success".to_owned(),
        message: message.to_owned(),
    };

    Ok(person_response(&mut cookies, data))
}

async fn create_genre(
    Extension(ref pool): Extension<PgPool>,
    form: Form<Genre>,
    mut cookies: Cookies,
)-> Result<GenreResponse, AppError> {

    let genre = form.0;
    let new_genre = db::add_genre(pool, genre).await?;
    let message = format!("Genre ajouté : {}", new_genre.name);

    let data = FlashData {
        kind: "success".to_owned(),
        message: message.to_owned(),
    };

    Ok(genre_response(&mut cookies, data))
}



#[debug_handler]
async fn create_partition (
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

// ***************************************************************************************
// handlers to update rows in DB tables
//


async fn update_genre(
    Extension(ref conn): Extension<DatabaseConnection>,
    Path(id): Path<i32>,
    form: Form<genre::Model>,
    mut cookies: Cookies,
)->  Result<GenreResponse, (StatusCode, &'static str)> {
    let model = form.0;
    genre::ActiveModel {
        id: Set(id),
        name: Set(model.name.to_owned()),
    }
        .save(conn)
        .await
        .expect("could not edit genre");

    let data = FlashData {
        kind: "success".to_owned(),
        message: "Genre successfully updated".to_owned(),
    };

    Ok(genre_response(&mut cookies, data))
}

async fn update_person(
      Extension(ref conn): Extension<DatabaseConnection>,
      Path(id): Path<i32>,
      form: Form<persons::Model>,
      mut cookies: Cookies,
)->  Result<PersonResponse, (StatusCode, &'static str)> {

    let model = form.0;
    persons::ActiveModel {
        id: Set(id),
        full_name: Set(model.full_name.to_owned()),
    }
        .save(conn)
        .await
        .expect("could not edit person");

    let data = FlashData {
        kind: "success".to_owned(),
        message: "Person successfully updated".to_owned(),
    };

    Ok(person_response(&mut cookies, data))
}

async fn update_partition() {
    todo!()
}

//**********************************************************************************************
// handlers to delete rows in DB
//

async fn delete_person(
    Extension(ref pool): Extension<PgPool>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
) -> Result<PersonResponse, AppError> {

    let del = db::delete_person(id, pool).await?;

    let data = FlashData {
        kind: "success".to_owned(),
        message: format!("Person successfully deleted: {}", del).to_owned(),
    };

    Ok(person_response(&mut cookies, data))
}

async fn delete_genre(
    Extension(ref pool): Extension<PgPool>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
) -> Result<PersonResponse, AppError> {

    let genre_nom = db::delete_genre(id, pool).await?;

    let data = FlashData {
        kind: "success".to_owned(),
        message: format!("Genre succcessfully deleted : {}", genre_nom).to_owned(),
    };

    Ok(genre_response(&mut cookies, data))
}

async fn delete_partition(
    Extension(ref pool): Extension<PgPool>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
) -> Result<PersonResponse, AppError>  {

    let partition_title = db::delete_partition(id, pool).await?;

    let data = FlashData {
        kind: "success".to_owned(),
        message: format!("Partition succcessfully deleted : {}", partition_title).to_owned(),
    };

    Ok(partition_response(&mut cookies, data))
}

//*****************************************************************************
// handlers to print list of rows of table (print to pdf or printer
//

async fn print_list_genres(
    Extension(ref templates): Extension<Tera>,
    Extension(ref conn): Extension<DatabaseConnection>,
    _cookies: Cookies,
)->  Result<Html<String>, (StatusCode, &'static str)> {

    let genres = SeaOrmGenre::find()
        .order_by(genre::Column::Name, Order::Asc)
        .all(conn)
        .await
        .unwrap();

    let title = "Liste des Genres";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("genres", &genres);

    let body = templates
        .render("list_genres.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error in list_genres.html.tera"))?;

    Ok(Html(body))
}

async fn print_list_persons(
    Extension(ref templates): Extension<Tera>,
    Extension(ref conn): Extension<DatabaseConnection>,
    _cookies: Cookies,
)->  Result<Html<String>, (StatusCode, &'static str)> {
    let persons = SeaOrmPerson::find().order_by(persons::Column::FullName, Order::Asc).all(conn).await.unwrap();
    let title = "Liste des Personnes";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("persons", &persons);

    let body = templates
        .render("list_users.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn print_list_partitions(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    _cookies: Cookies,
)->  Result<Html<String>, (StatusCode, &'static str)> {

    let show_partitions = db::list_show_partitions(pool).await.unwrap();
    let title = "liste des partitions";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("partitions", &show_partitions);

    let body = templates
        .render("list_partitions.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error in list_partitions.html.tera"))?;

    Ok(Html(body))
}

//*******************************************************************************************
// handlers to find rows from DB on different criteria
//

#[derive(Deserialize, Serialize, Debug, Clone,)]
pub struct Demande {
    pub name : String,
}

///
/// find_genre_by_name
///
/// returns list genre page with genre found
///
async fn find_genre_by_name(
    Extension(ref templates): Extension<Tera>,
    Extension(ref conn): Extension<DatabaseConnection>,
    form: Form<Demande>,
    _cookies: Cookies,
)->  Result<Html<String>, (StatusCode, &'static str)> {

    let demande = form.0;
    tracing::debug!("name : {:?}", demande);

    let name = demande.name;

    let genres: Vec<genre::Model> = SeaOrmGenre::find()
        .filter(genre::Column::Name.contains(&name))
        .all(conn).await.unwrap();

    let title = "Genre(s) trouvé(s)";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("genres", &genres);

    let body = templates
        .render("genres.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

///
/// find_person_by_name
///
/// returns list musicians page with musician found
///
async fn find_person_by_name(
    Extension(ref templates): Extension<Tera>,
    //Extension(ref conn): Extension<DatabaseConnection>,
    Extension(ref pool): Extension<PgPool>,
    form: Form<Demande>,
    _cookies: Cookies,
)->  Result<Html<String>, (StatusCode, &'static str)> {

    let demande = form.0;
    tracing::debug!("name : {:?}", demande);

    let name = demande.name;
/*
    let persons: Vec<persons::Model> = SeaOrmPerson::find()
        .filter(persons::Column::FullName.contains(&name))
        .all(conn).await.unwrap();
*/
    let person = db::find_person_by_name(name, pool).await;
    match person {
        Ok(person) => {
            let title = "Personne(s) trouvée(s)";

            let mut persons: Vec<Person> = Vec::new();
            persons.push(person);

            let mut ctx = tera::Context::new();
            ctx.insert("title", &title);
            ctx.insert("persons", &persons);

            let body = templates
                .render("persons.html.tera", &ctx)
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;
            Ok(Html(body))
        }
        Err(RowNotFound) => {
            let mut ctx = tera::Context::new();
            ctx.insert("data", "personne");

            let body = templates
                .render("error/void.html.tera", &ctx)
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;
            Ok(Html(body))
        }
    }


}

async fn find_partition_title(
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

async fn find_partition_genre(
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

async fn find_partition_author(
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

//******************************************************************************
// functions for practical use
//

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
        let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
    tracing::debug!("signal ctrl_c reçu");
}




