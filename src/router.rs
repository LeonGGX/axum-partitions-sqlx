// src/router.rs

use axum::{
    routing::{get, post},
    Router,
};
use axum_flash::Key;

use crate::handlers::{
    partitions_hdl::*,
    musicians_hdl::*,
    genres_hdl::*,
    helpers_hdl::*,
    login_hdl::*,
    sign_up_hdl::*,
};
use crate::handlers::list_users_hdl::print_list_users_hdl;

pub fn router() -> Router {
    Router::new()
        .fallback(get(handler_404))
        .route("/", get(root))
        .nest("/persons", musicians_routes())
        .nest("/genres", genres_routes())
        .nest("/partitions", partitions_routes())
        .nest("/auth", authentication_routes())
        .nest("/about", get(about))

}

pub fn musicians_routes() -> Router {
    Router::new()
        .route("/", get(list_persons_hdl))
        .route("/add", post(create_person_hdl))
        .route("/:id", post(update_person_hdl))
        .route("/delete/:id", post(delete_person_hdl))
        .route("/print", get(print_list_persons_hdl))
        .route("/find", post(find_person_by_name_hdl))

}

pub fn genres_routes() -> Router {
    Router::new()
        .route("/", get(list_genres_hdl))
        .route("/add", post(create_genre_hdl))
        .route("/:id", post(update_genre_hdl))
        .route("/delete/:id", post(delete_genre_hdl))
        .route("/print", get(print_list_genres_hdl))
        .route("/find", post(find_genre_by_name_hdl))
}

pub fn partitions_routes() -> Router {
    Router::new()
        .route("/", get(list_partitions_hdl))
        .route("/add", post(create_partition_hdl))
        .route("/:id", post(update_partition_hdl))
        .route("/delete/:id", post(delete_partition_hdl))
        .route("/print", get(print_list_partitions_hdl))
        .route("/find/title", post(find_partition_title_hdl))
        .route("/find/genre", post(find_partition_genre_hdl))
        .route("/find/author", post(find_partition_author_hdl))

}

pub fn authentication_routes() -> Router {
    // axum-flash
    //let key = Key::generate();
    Router::new()
        .route("/signup", get(get_sign_up_hdl).post(sign_up_hdl))
        .route("/login", get(login_form_hdl).post(login_hdl))
        .route("/users", get(print_list_users_hdl))
        // axum-flash
       //.layer(axum_flash::layer(key).with_cookie_manager())
}
