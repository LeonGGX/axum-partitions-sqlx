//! src/handlers/list_users_hdl.rs

use crate::db::users::list_users;
use crate::AppError;
use axum::extract::Extension;
use axum::response::Html;
use sqlx::PgPool;
use tera::Tera;
use tower_cookies::Cookies;

pub async fn print_list_users_hdl(
    Extension(ref templates): Extension<Tera>,
    Extension(ref pool): Extension<PgPool>,
    _cookies: Cookies,
) -> Result<Html<String>, AppError> {
    let users = list_users(pool).await?;
    //.map_err(|e|AppError::Sqlx(e));

    let title = "Liste des Utilisateurs";

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("users", &users);

    let body = templates
        .render("list_users.html", &ctx)
        .map_err(|err| AppError::Tera(err))?;

    Ok(Html(body))
}
