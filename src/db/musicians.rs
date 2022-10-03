//! src/db/musicians.rs

use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

use crate::models::musician::Person;

//*******************************************************************************************
// CRUD Operations on persons - musicians
//
pub async fn add_person(pool: &PgPool, pers: Person) -> sqlx::Result<Person> {
    let mut tx = pool.begin().await?;
    let person = sqlx::query(
        "INSERT INTO persons (full_name)
                VALUES ( $1 )
                RETURNING id, full_name;",
    )
    .bind(&pers.full_name)
    .map(|row: PgRow| Person {
        id: row.get(0),
        full_name: row.get(1),
    })
    .fetch_one(&mut tx)
    .await?;
    tx.commit().await?;

    tracing::info!("db : person added : {:?}", &person);
    Ok(person)
}

pub async fn update_person(id: i32, person_name: String, pool: &PgPool) -> sqlx::Result<Person> {
    let person =
        sqlx::query("UPDATE persons SET full_name = $1 WHERE id = $2 RETURNING id, full_name;")
            .bind(&person_name)
            .bind(id)
            .map(|row: PgRow| Person {
                id: row.get(0),
                full_name: row.get(1),
            })
            .fetch_one(pool)
            .await?;

    tracing::info!("db : Person updated : {:?}", &person);
    Ok(person)
}

pub async fn delete_person(id: i32, pool: &PgPool) -> sqlx::Result<String> {
    let pers = find_person_by_id(id.clone(), pool).await?;
    let name = pers.full_name;

    let _res = sqlx::query("DELETE FROM persons WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    tracing::info!("db : Person deleted : {}", &name);

    Ok(name)
}

//**********************************************************************************
// Fonctions de recherche d'enregistrements sur base de critères : nom, genre, titre, ...
//

///
/// find person by id
/// used as help function for others
///
pub async fn find_person_by_id(id: i32, pool: &PgPool) -> sqlx::Result<Person> {
    let person = sqlx::query("SELECT * FROM persons WHERE id = $1;")
        .bind(id)
        .map(|row: PgRow| Person {
            id: row.get(0),
            full_name: row.get(1),
        })
        .fetch_one(pool)
        .await?;

    tracing::info!("db : Personne trouvée : {}", &person.full_name);
    Ok(person)
}

///
/// find person by name
/// returns author by name
///
pub async fn find_person_by_name(full_name: String, pool: &PgPool) -> sqlx::Result<Vec<Person>> {
    let mut name = full_name.clone();
    name.push('%');

    let select_query = sqlx::query(
        "SELECT * FROM persons \
                         WHERE full_name LIKE $1",
    );
    let person = select_query
        .bind(name)
        .map(|row: PgRow| Person {
            id: row.get("id"),
            full_name: row.get("full_name"),
        })
        .fetch_all(pool)
        .await?;

    Ok(person)
}

///
/// Returns a list of musicians
/// under the form of a Vec<Person>
/// or a sqlx Error
///
pub async fn list_persons(pool: &PgPool) -> sqlx::Result<Vec<Person>> {
    //let mut persons: Vec<Person> = Vec::new();
    let recs = sqlx::query("SELECT id, full_name FROM persons ORDER BY full_name;")
        .map(|row: PgRow| Person {
            id: row.get("id"),
            full_name: row.get("full_name"),
        })
        .fetch_all(pool)
        .await?;

    Ok(recs)
}
