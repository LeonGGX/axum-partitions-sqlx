//! src/handlers/helpers_hdl.rs

use std::io;
use axum::body::Bytes;
use axum::extract::Extension;
use axum::http::{header, StatusCode, Uri};
use axum::response::{Html, IntoResponse};
use tera::Tera;
use crate::AppError;

// Il faut une fonction root qui ramène à la racine
// sinon problème. Sauf si on utilise Redirect
//
pub async fn root(Extension(ref templates): Extension<Tera>,) -> Result<Html<String>, AppError> {
    let title = "Start";
    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    let body = templates
        .render("start.html", &ctx)
        .map_err(|e| AppError::Tera(e))?;
    Ok(Html(body))
}

pub async fn about(
    Extension(ref templates): Extension<Tera>,
) -> Result<Html<String>, AppError> {
    let title = "A propos de ...";
    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    let body = templates
        .render("about.html", &ctx)
        .map_err(|e| AppError::Tera(e))?;
    Ok(Html(body))
}


pub async fn handler_404(
    Extension(ref templates): Extension<Tera>,
    uri: Uri,
) -> Result<Html<String>, (StatusCode, &'static str)> {

    let title = "Erreur de routing";
    let origin = uri.path();
    let mut ctx = tera::Context::new();
    ctx.insert("title", title);
    ctx.insert("uri", origin);
    let body = templates
        .render("error/404.html", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error in 404.html"))?;
    Ok(Html(body))
}


pub async fn shutdown_signal(){
    tokio::signal::ctrl_c()
        .await
        .expect("expect tokio signal ctrl-c");
    println!("signal shutdown");
}


pub async fn favicon() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/png")],
        Bytes::from_static(include_bytes!(
            "D:\\Programmation\\Rust\\mes_programmes\\axum-jwt\\static\\images\\rust-logo-white.png")),
    )
}


