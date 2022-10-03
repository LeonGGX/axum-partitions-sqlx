//! src/auth/session.rs

use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
use rand_chacha::ChaCha8Rng;
use rand_core::{OsRng, RngCore};

use axum::http;
use axum::response::IntoResponse;

// Ascendig Creation
use axum_database_sessions::{
    AxumPgPool, AxumPgSessionStore, AxumSession, AxumSessionConfig, Key, SessionError,
};
use rand::Rng;

//
use async_session;
use async_sqlx_session::PostgresSessionStore;
use axum_sessions::{
    async_session::MemoryStore,
    extractors::{ReadableSession, WritableSession},
    SessionLayer,
};

use sqlx::PgPool;
use uuid::Uuid;

use crate::db;
use tower_cookies::Cookie;

use crate::error::{LoginError, SignupError};
use crate::flash::signup_response;
use crate::models::user::{NewUser, NewUserName};
use crate::utils::auth_utils::{hash_password_argon2, hash_password_pbkdf2, parse};


pub(crate) async fn login_session(
    database: &PgPool,
    session: AxumSession<AxumPgPool>,
    username: String,
    password: String,
) -> Result<AxumSession<AxumPgPool>, LoginError> {
    const LOGIN_QUERY: &str = "SELECT id, password_hash FROM users WHERE name = $1;";

    let row: Option<(Uuid, String)> = sqlx::query_as(LOGIN_QUERY)
        .bind(username)
        .fetch_optional(database)
        .await
        .unwrap();

    let (user_id, hashed_password) = if let Some(row) = row {
        row
    } else {
        return Err(LoginError::UserDoesNotExist);
    };

    // Verify password against PHC string
    let parsed_hash = PasswordHash::new(&hashed_password).unwrap();
    if let Err(_err) = Pbkdf2.verify_password(password.as_bytes(), &parsed_hash) {
        return Err(LoginError::WrongPassword);
    }
    //session.clear_all().await;
    session.set("id", user_id).await;
    Ok(session)
}

///
/// Function to treat data from a new user
/// generates a hashed password
/// adds the user to the DB
/// opens a session
/// returns the Uuid of the new user or SignupError
///
pub(crate) async fn signup_session(
    pool: &PgPool,
    session: AxumSession<AxumPgPool>,
    username: &str,
    password: &str,
    role: &str,
) -> Result<Uuid, SignupError> {
    let string_username: &String = &username.to_string();
    tracing::info!("username = {}", string_username);

    // parse vérifie si le nom d'utilisateur est correct
    // on vérifie ensuite si le futur utilisateur existe dans la DB afin d'éviter les doublons
    // on crée un nouvel utilisateur NewUser avec un mot de passe crypté
    // on ajoute le nouvel utilisateur à la DB
    // on renvoie vers la page signup avec le message de création du nouvel utilisateur
    if let Ok(name) = parse(string_username) {
        if let Ok(_user) = db::users::find_user_by_name(name, &pool).await {
            return Err(SignupError::UsernameExists);
        } else {
            let name = NewUserName::parse(username.to_string()).unwrap();
            // fonction de hash ici
            let password = hash_password_pbkdf2(password.to_string()).await.unwrap();
            // *********************
            let new_user = NewUser {
                name,
                password,
                role: role.to_string(),
            };
            let added_user = db::users::add_user(&new_user, &pool).await.unwrap();

            tracing::info!("utilisateur ajouté : {:?}", added_user);

            let user_id = added_user.id;
            session.clear_all().await;
            session.set("user-id", user_id).await;
            return Ok(session.get("user-id").await.unwrap());
        }
    } else {
        tracing::info!("Erreur de validation du nom d'utilisateur");
        return Err(SignupError::InvalidUsername);
    }
}

///
/// Creates a new AscendingCreation session
/// with a Postgresql table in persons DB
/// returns AxumPgSessionStore or a SessionError
///
#[allow(dead_code)]
pub(crate) async fn new_ascd_creation_sqlx_session(
    pool: &PgPool,
) -> Result<AxumPgSessionStore, SessionError> {
    let key = Key::generate();
    // axum_sqlx_session
    let session_config = AxumSessionConfig::default()
        //.with_table_name("persons.sessions")
        .with_key(key.clone());
    tracing::debug!("AxumSessionConfig : {:?}", session_config);

    let session_pool = Option::from(AxumPgPool::from(pool.clone()));
    let session_store = AxumPgSessionStore::new(session_pool, session_config);

    let init = session_store.initiate().await;
    return match init {
        Ok(_) => Ok(session_store),
        Err(e) => Err(e),
    };
    //Ok(session_store)
}

///
/// Creates a new axum_sessions session
/// with a Postgresql table in persons DB
/// returns a SessionLayer
///
#[allow(dead_code)]
pub(crate) async fn new_axum_sqlx_session(pool: PgPool) -> SessionLayer<MemoryStore> {
    // doesn't work due to old sqlx version in crate axum_sqlx_session
    //let store = PostgresSessionStore::from_client(pool);
    //store.migrate().await.expect("Couldn't migrate session store");

    let store = MemoryStore::new();

    let secret = rand::thread_rng().gen::<[u8; 128]>();
    let session_layer = SessionLayer::new(store, &secret);
    session_layer
}

