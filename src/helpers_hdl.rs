// src/handlers/helpers_hdl.rs

use axum::extract::Extension;
use axum::http::{StatusCode, Uri};
use axum::response::Html;
use tera::Tera;
use tokio::signal;
use crate::AppError;

// Il faut une fonction root qui ramène à la racine
// sinon problème. Sauf si on utilise Redirect
//
pub async fn root(Extension(ref templates): Extension<Tera>,) -> Result<Html<String>, AppError> {

    let title = "Start";
    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    let body = templates
        .render("start.html.tera", &ctx)
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
        .render("about.html.tera", &ctx)
        .map_err(|e| AppError::Tera(e))?;

    Ok(Html(body))
}


pub async fn handler_404(
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


pub async fn shutdown_signal() {
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
