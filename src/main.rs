// src/main.rs

mod db;
mod flash;
mod model;
mod error;
mod handlers;

use axum::{
    http::{StatusCode,},
    routing::{get, post, get_service},
    Router,
    AddExtensionLayer,
};

use std::{env, net::SocketAddr};
use std::str::FromStr;

use tera::Tera;

use tower::ServiceBuilder;
use tower_cookies::{CookieManagerLayer,};
use tower_http::services::ServeDir;

use crate::error::AppError;
use crate::handlers::partitions_hdl::*;
use crate::handlers::genres_hdl::*;
use crate::handlers::musicians_hdl::*;
use crate::handlers::helpers_hdl::*;

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
        .route("/partitions/:id", post(update_partition))
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





