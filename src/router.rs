//! src/router.rs

//**********************************************************************************
// Normalement un seul fallback par route
// il est peut-être possible de prévoir un fallback pour musician_routes et un autre
// pour genres_routes, ...
// à condition d'enlever celui de la fonction router()
// ?? A tester
//***********************************************************************************

use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::{
    partitions_hdl::*,
    musicians_hdl::*,
    genres_hdl::*,
    helpers_hdl::*,
    login_hdl::*,
    sign_up_hdl::*,
    list_users_hdl::*,
};

///
/// Fonction qui construit toutes les routes à partir des fonctions
/// qui construisent les ensembles de routes pour une même catégorie
///
pub fn router() -> Router {
    Router::new()
        .fallback(get(handler_404))
        .route("/", get(root))
        .nest("/persons", musicians_routes())
        .nest("/genres", genres_routes())
        .nest("/partitions", partitions_routes())
        .nest("/auth", authentication_routes())
        .nest("/about", get(about))
        .route("/favicon.png", get(favicon))
}

///
/// gère les routes vers les pages de gestion des musiciens
/// la route "/" correspond à "/persons", "/add" correspond à "/persons/add"
///
pub fn musicians_routes() -> Router {
    Router::new()
        //.route("/", get(askama_list_persons_hdl))
        .route("/", get(list_persons_hdl))
        .route("/add", post(create_person_hdl))
        .route("/:id", post(update_person_hdl))
        .route("/delete/:id", post(delete_person_hdl))
        .route("/print", get(print_list_persons_hdl))
        .route("/find", post(find_person_by_name_hdl))
}

///
/// gère les routes vers les pages de gestion des genres
/// la route "/" correspond à "/genres"
///
pub fn genres_routes() -> Router {
    Router::new()
        .route("/", get(list_genres_hdl))
        .route("/add", post(create_genre_hdl))
        .route("/:id", post(update_genre_hdl))
        .route("/delete/:id", post(delete_genre_hdl))
        .route("/print", get(print_list_genres_hdl))
        .route("/find", post(find_genre_by_name_hdl))
}

///
/// gère les routes vers les pages de gestion des partitions
/// la route "/" correspond à "/partitions"
///
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

///
/// gère les routes vers les pages d'authentification
/// la route "/" correspond à "/auth"
///
pub fn authentication_routes() -> Router {
    Router::new()
        .route("/signup", get(get_sign_up_hdl).post(sign_up_hdl))
        .route("/login", get(login_form_hdl).post(login_hdl))
        .route("/users", get(print_list_users_hdl))
}
