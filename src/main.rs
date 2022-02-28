//! src/main.rs

mod flash;
mod model;
mod error;
mod handlers;
mod auth;
mod router;
mod db;


use axum::{
    http::{StatusCode,},
    routing::{get_service, },
    AddExtensionLayer,
};

use std::{env, net::SocketAddr};
use std::str::FromStr;

use tera::Tera;

use tower::ServiceBuilder;
use tower_cookies::{CookieManagerLayer,};
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::error::AppError;

use crate::handlers::helpers_hdl::*;

use crate::router::router;

use crate::db::connect::create_pg_pool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "axum_jwt=debug,tower_http=debug")
    }
    tracing_subscriber::fmt::init();

    env::set_var( "JWT_SECRET", "secret");

    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{}:{}", host, port);

    // ici utilisation de sqlx
    let pool = create_pg_pool(&db_url).await?;

    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"))
        .expect("Tera initialization failed");

    let app =
        router()
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
                .layer(TraceLayer::new_for_http())
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





