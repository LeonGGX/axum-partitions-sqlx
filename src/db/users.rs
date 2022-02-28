//! src/db/users.rs

use sqlx::PgPool;
use uuid::Uuid;
use sha3::Digest;

use crate::model::{NewUser, User};

//******************************************************************************************
// Authentication functions
//

pub async fn find_user_by_credentials(user_name: String, user_password : String, pool: &PgPool) -> sqlx::Result<User> {

    let password_hash = sha3::Sha3_256::digest(user_password.as_bytes());
    let password_hash = format!("{:x}", password_hash);

    let row = sqlx::query!(
        r#"
    SELECT * FROM users
    WHERE user_name = $1 AND password_hash = $2
        "#,
    user_name,
    password_hash,)
        .fetch_one(pool)
        .await?;

    let user = User{
        user_id: row.user_id,
        user_name: row.user_name,
        password_hash: row.password_hash,
        role: row.role.unwrap(),
    };
    Ok(user)
}

pub async fn add_user(new_user: &NewUser,
                      pool: &PgPool ) -> sqlx::Result<User> {

    let password_hash = sha3::Sha3_256::digest(new_user.password.as_bytes());
    let password_hash = format!("{:x}", password_hash);
    let uuid = Uuid::new_v4();

    let row = sqlx::query!(
            r#"
            INSERT INTO users (user_id, user_name, password_hash, role)
            VALUES ($1, $2, $3, $4)
            RETURNING user_id, user_name, password_hash, role
            "#,
            uuid,
            new_user.user_name.as_ref(),
            password_hash,
            new_user.role,
        )
        .fetch_one(pool)
        .await?;

    let user = User{
        user_id: row.user_id,
        user_name: row.user_name,
        password_hash: row.password_hash,
        role: row.role.unwrap(),
    };
    Ok(user)
}
