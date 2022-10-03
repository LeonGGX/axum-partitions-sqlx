//! src/main.rs

mod auth;
mod db;
mod error;
mod flash;
mod handlers;
mod models;
mod router;
mod globals;
mod utils;

use std::str::FromStr;
use std::{env, net::SocketAddr};

use axum::{http::StatusCode, Extension};
use axum_flash::Key;

use axum_database_sessions::{ AxumSessionLayer, };

use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tera::Tera;

use crate::auth::session::{new_ascd_creation_sqlx_session, new_axum_sqlx_session};
use crate::db::connect::create_pg_pool;
use crate::error::AppError;
use crate::handlers::helpers_hdl::*;
use crate::router::router;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "axum_jwt=debug,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{}:{}", host, port);

    // ici utilisation de sqlx
    let pool = create_pg_pool(&db_url).await?;

    // Tera templates
    let templates = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    // axum-flash
    // vient de tower_cookies::Key
    let key = Key::generate();

    // Ascending session
    let session_store = new_ascd_creation_sqlx_session(&pool).await;

    match session_store {
        Ok(ref s) => {
            tracing::info!("session créée");
            let count = s.clone().count().await.unwrap();
            let client = s.client.clone().unwrap();
            tracing::info!("client de sessions : {:?}", &client);
            tracing::info!("nombre de sessions : {}", count);
        }
        Err(err) => {
            tracing::error!("erreur : {:?}", err);
            panic!("erreur dans session");
        }
    }
    let session = session_store.unwrap();

    let app =
        // fonction qui vient de 'router.rs' et qui construit toutes les routes
        router()
        .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CookieManagerLayer::new())
                    .layer(Extension(pool))
                    .layer(Extension(templates)))
                    .layer(axum_flash::layer(key).with_cookie_manager())
                    .layer(AxumSessionLayer::new(session));

    let addr = SocketAddr::from_str(&server_url).unwrap();
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
