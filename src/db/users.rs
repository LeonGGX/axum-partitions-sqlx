//! src/db/users.rs

use sqlx::{PgPool, Row, /*types::Uuid */};
use uuid::Uuid;
use sha3::Digest;
use sqlx::postgres::PgRow;

use crate::models::user::{NewUser, User};

//******************************************************************************************
// Authentication functions
//
pub async fn find_user_by_name(name: String, pool: &PgPool) -> sqlx::Result<User>{

    let row = sqlx::query!(
        r#"
    SELECT * FROM users WHERE name = $1
        "#,
    name,
    )
        .fetch_one(pool)
        .await?;
    let user = User{
        id: row.id,
        name: row.name,
        password_hash: row.password_hash,
        role: row.role.unwrap(),
    };
    Ok(user)

}
#[allow(dead_code)]
pub async fn find_user_by_credentials(user_name: String, user_password : String, pool: &PgPool) -> sqlx::Result<User> {

    let password_hash = sha3::Sha3_256::digest(user_password.as_bytes());
    let password_hash = format!("{:x}", password_hash);

    let row = sqlx::query!(
        r#"
    SELECT * FROM users
    WHERE name = $1 AND password_hash = $2
        "#,
    user_name,
    password_hash,)
        .fetch_one(pool)
        .await?;

    let user = User{
        id: row.id,
        name: row.name,
        password_hash: row.password_hash,
        role: row.role.unwrap(),
    };
    Ok(user)
}
#[allow(dead_code)]
pub async fn find_user_by_id(id: Uuid, pool: &PgPool) -> sqlx::Result<User> {

    let row = sqlx::query!(
        r#"
    SELECT * FROM users
    WHERE id = $1
        "#,
        id,)
        .fetch_one(pool)
        .await?;

    let user = User{
        id: row.id,
        name: row.name,
        password_hash: row.password_hash,
        role: row.role.unwrap(),
    };
    Ok(user)
}

/*********************************************************************************
CRUD FUNCTIONS
 */
///
/// Adds a user to the users table
/// a hashed password must be set before entering the function
///
pub async fn add_user(new_user: &NewUser,
                      pool: &PgPool ) -> sqlx::Result<User> {
    // si on fait le hash dans le handler signup alors on enlève les deux lignes après
    //let password_hash = sha3::Sha3_256::digest(new_user.password.as_bytes());
    //let password_hash = format!("{:x}", password_hash);
    let uuid = Uuid::new_v4();

    let row = sqlx::query!(
            r#"
            INSERT INTO users (id, name, password_hash, role)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, password_hash, role
            "#,
            uuid,
            new_user.name.as_ref(),
            new_user.password,
            new_user.role,
        )
        .fetch_one(pool)
        .await?;

    let user = User{
        id: row.id,
        name: row.name,
        password_hash: row.password_hash,
        role: row.role.unwrap(),
    };
    Ok(user)
}

/*****************************************************************************
DISPLAY FUNCTIONS
 */

pub async fn list_users(pool: &PgPool) -> sqlx::Result<Vec<User>> {

    let pwd = "secret".to_string();
    let mut list_safe_users: Vec<User> = Vec::new();
    let rows = sqlx::query(
        r#"SELECT user_id, user_name, password_hash, role FROM users ORDER BY user_name"#
    )
        .map(|row: PgRow| User{
            id: row.get("user_id"),
            name: row.get("user_name"),
            password_hash: row.get("password_hash"),
            role: row.get("role")
        })
        .fetch_all(pool)
        .await?;

    for mut row in rows {
        row.password_hash = pwd.clone();
        list_safe_users.push(User{
            id: row.id,
            name: row.name,
            password_hash: row.password_hash,
            role: row.role,
        });
    }
    Ok(list_safe_users)
}
