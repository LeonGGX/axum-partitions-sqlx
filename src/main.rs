//! src/main.rs

mod flash;
mod models;
mod error;
mod handlers;
mod auth;
mod router;
mod db;
//mod my_askama;
mod globals;


use axum::{
    http::{StatusCode,},
    Extension,
};
use axum_flash::Key;

//use axum_database_sessions::{AxumSession, AxumSessionConfig, AxumSessionLayer, AxumDatabasePool};
//use axum_sessions_auth::{AuthSession, AuthSessionLayer, Authentication};

use std::{env, net::SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use rand_chacha::ChaCha8Rng;
use rand_core::{OsRng, RngCore, SeedableRng};

use tera::Tera;
use tokio::sync::Mutex;

use tower::{ServiceBuilder,};
use tower_cookies::{CookieManagerLayer,};
use tower_http::{trace::TraceLayer, };
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::error::AppError;
use crate::handlers::helpers_hdl::*;
use crate::router::router;
use crate::db::connect::create_pg_pool;


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "axum_jwt=debug,tower_http=info".into()),
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
    let key = Key::generate();

    // pour les sessions
    let random = rand_chacha::ChaCha8Rng::seed_from_u64(OsRng.next_u64());

    let app =
        // fonction qui vient de 'router.rs' et qui construit toutes les routes
        router()
        .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CookieManagerLayer::new())
                    .layer(Extension(pool))
                    .layer(Extension(templates)))
                    // axum-flash
                    .layer(axum_flash::layer(key).with_cookie_manager())
                    .layer(Extension(Arc::new(Mutex::new(random))));

    let addr = SocketAddr::from_str(&server_url).unwrap();
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}







