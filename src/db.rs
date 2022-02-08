// src/db.rs

use sqlx::postgres::{PgPool, PgRow,};
use sqlx::{Row, Result, Error};
use tracing::log::Record;
use crate::model::{Person, ShowPartition, Genre, Partition};

///
/// Open a connection to a database
///
pub async fn create_pg_pool(db_url: &str) -> sqlx::Result<PgPool> {

    let pool = PgPool::connect(db_url).await?;
    Ok(pool)
}



pub async fn list_persons(pool: &PgPool) -> sqlx::Result<Vec<Person>> {

    let mut persons: Vec<Person> = Vec::new();
    let recs: Vec<Person> =
        sqlx::query("SELECT id, full_name FROM persons ORDER BY full_name;")
            .map(|row: PgRow| Person {
                id: row.get("id"),
                full_name: row.get("full_name"),
            })
            .fetch_all(pool)
            .await?;

    for rec in recs {
        persons.push(Person {
            id: rec.id,
            full_name: rec.full_name,
        });
    }
    Ok(persons)

    /*
    let persons = sqlx::query_as!(Person,
    "SELECT id, full_name FROM persons ORDER BY full_name")
        .fetch_all(pool)
        .await?;

    Ok(persons)
    */
}

pub async fn list_genres(pool: &PgPool) -> anyhow::Result<Vec<Genre>> {

    let genres: Vec<Genre> =
        sqlx::query("SELECT id, name FROM genres ORDER BY name;")
            .map(|row: PgRow| Genre {
                id: row.get(0),
                name: row.get(1),
            })
            .fetch_all(pool)
            .await?;
    Ok(genres)
}

pub async fn list_show_partitions(pool: &PgPool) -> anyhow::Result<Vec<ShowPartition>> {

    let rep: Vec<ShowPartition>  =
        sqlx::query("
    SELECT partitions.id, partitions.title, persons.full_name, genres.name
    FROM partitions
    INNER JOIN persons
    ON partitions.person_id = persons.id
    INNER JOIN genres
    ON partitions.genre_id = genres.id
        "
        ).map(|row: PgRow| ShowPartition {
            id: row.get(0),
            title: row.get(1),
            full_name: row.get(2),
            name: row.get(3),
        })
            .fetch_all(pool)
            .await?;
    Ok(rep)
}

pub async fn show_one_partition(partition: Partition, pool: &PgPool) -> sqlx::Result<ShowPartition> {

    let show_partition = sqlx::query(
        "
    SELECT partitions.id, partitions.title, persons.full_name, genres.name
    FROM partitions
    INNER JOIN persons
    ON partitions.person_id = persons.id
    INNER JOIN genres
    ON partitions.genre_id = genres.id
    WHERE partitions.title = $1
        "
    )
        .bind(partition.title)
        .map(|row: PgRow| ShowPartition {
        id: row.get(0),
        title: row.get(1),
        full_name: row.get(2),
        name: row.get(3),
    })
        .fetch_one(pool)
        .await?;

    Ok(show_partition)
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

    tracing::info!(
        "DB : Personne trouvée : {}",
        &person.full_name
    );
    Ok(person)
}

///
/// find person by name
/// returns author by name
///
pub async fn find_person_by_name(
    full_name: String,
    pool: &PgPool) -> sqlx::Result<Person> {

    let select_query = sqlx::query("SELECT * FROM persons WHERE full_name = $1");
    let person =
        select_query
            .bind(full_name)
            .map(|row: PgRow| {
                Person {
                    id: row.get("id"),
                    full_name: row.get("full_name"),
                }
            })
            .fetch_one(pool)
            .await?;

    tracing::info!(
        "DB find_person_by_name : Personne trouvée (nom) : {}",
        &person.full_name
    );

    Ok(person)
}

pub async fn find_genre_by_id(id: i32, pool: &PgPool) -> sqlx::Result<Genre> {

    let genre = sqlx::query("SELECT * FROM genres WHERE id = $1;")
        .bind(id)
        .map(|row: PgRow| Genre {
            id: row.get(0),
            name: row.get(1),
        })
        .fetch_one(pool)
        .await?;

    tracing::info!(
        "DB : Genre trouvé : {}",
        &genre.name
    );
    Ok(genre)
}

pub async fn find_genre_by_name(name: String, pool: &PgPool) -> sqlx::Result<Genre> {

    let genre = sqlx::query("SELECT * FROM genres WHERE name = $1;")
        .bind(name)
        .map(|row: PgRow| Genre {
            id: row.get("id"),
            name: row.get("name"),
        })
        .fetch_one(pool)
        .await?;

    tracing::info!(
        "DB : Genre trouvé (nom) : {}",
        &genre.name
    );

    Ok(genre)
}

pub async fn find_partition_by_id(
    id: i32,
    pool: &PgPool
) -> sqlx::Result<Partition> {

    let partition = sqlx::query("SELECT * FROM partitions WHERE id = $1;")
        .bind(id)
        .map(|row: PgRow| Partition {
            id: row.get("id"),
            title: row.get("title"),
            person_id: row.get("person_id"),
            genre_id: row.get("genre_id"),
        })
        .fetch_one(pool)
        .await?;

    tracing::info!(
        "DB : partition trouvée (titre) : {}",
        &partition.title
    );
    Ok(partition)
}

pub async fn find_partition_by_title(
    title: String,
    pool: &PgPool) -> sqlx::Result<Partition>{

    let partition = sqlx::query("SELECT * FROM partitions WHERE title = $1;")
        .bind(title)
        .map(|row: PgRow| Partition {
            id: row.get("id"),
            title: row.get("title"),
            person_id: row.get("person_id"),
            genre_id: row.get("genre_id"),
        })
        .fetch_one(pool)
        .await?;

    tracing::info!(
        "DB : partition trouvée (titre) : {}",
        &partition.title
    );
    Ok(partition)
}

pub async fn find_partition_by_genre(
    genre_name: String, pool: &PgPool) -> sqlx::Result<Vec<Partition>> {

    let genre = find_genre_by_name(genre_name.clone(), pool).await?;
    let genre_id = genre.id.unwrap();

    let partitions = sqlx::query("SELECT * FROM partitions WHERE genre_id = $1;")
        .bind(genre_id)
        .map(|row: PgRow| Partition {
            id: row.get("id"),
            title: row.get("title"),
            person_id: row.get("person_id"),
            genre_id: row.get("genre_id"),
        })
        .fetch_all(pool)
        .await?;

    tracing::info!(
        "DB : partition(s) trouvée(s) pour genre : {}",
        &genre_name
    );
    Ok(partitions)
}

pub async fn find_partition_by_author(author_name: String, pool: &PgPool) -> sqlx::Result<Vec<Partition>> {

    let author = find_person_by_name(author_name.clone(), pool).await?;
    let author_id = author.id;

    let partitions = sqlx::query("SELECT * FROM partitions WHERE person_id = $1;")
        .bind(author_id)
        .map(|row: PgRow| Partition {
            id: row.get("id"),
            title: row.get("title"),
            person_id: row.get("person_id"),
            genre_id: row.get("genre_id"),
        })
        .fetch_all(pool)
        .await?;

    tracing::info!(
        "DB : partition(s) trouvée(s) pour auteur : {}",
        &author_name
    );
    Ok(partitions)
}


pub async fn add_person(
    pool: &PgPool,
    pers: Person) -> sqlx::Result<Person> {

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

    tracing::info!("DB : person added : {:?}", &person);
    Ok(person)
}

pub async fn update_person(
    id: i32,
    update_person: Person,
    pool: &PgPool,
) -> anyhow::Result<Person> {

    let mut tx = pool.begin().await.unwrap();
    let person = sqlx::query(
        "UPDATE persons \
                                        SET full_name = $1, \
                                        WHERE id = $2 \
                                        RETURNING id, full_name;",
    )
    .bind(&update_person.full_name)
    .bind(id)
    .map(|row: PgRow| Person {
        id: row.get(0),
        full_name: row.get(1),
    })
    .fetch_one(&mut tx)
    .await?;

    tx.commit().await?;

    tracing::info!("DB : Person updated : {:?}", &person);
    Ok(person)
}

pub async fn delete_person(
    id: i32,
    pool: &PgPool) -> sqlx::Result<String> {

    let pers = find_person_by_id(id.clone(), pool).await?;
    let name = pers.full_name;

    let res = sqlx::query("DELETE FROM persons WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    tracing::info!("DB : Person deleted : {}", &name);

    Ok(name)
}

pub async fn add_genre(
    pool: &PgPool,
    genre: Genre) -> sqlx::Result<Genre> {

    let mut tx = pool.begin().await?;
    let rec = sqlx::query(
        "INSERT INTO genres (name)
                VALUES ( $1 )
                RETURNING id, name;"
    )
        .bind(&genre.name)
        .map(|row: PgRow| Genre {
            id: row.get(0),
            name: row.get(1),
        })
        .fetch_one(&mut tx)
        .await?;
    tx.commit().await?;

    tracing::info!("DB : genre added : {:?}", &rec);
    Ok(rec)
}

pub async fn update_genre() {
    todo!()
}

pub async fn delete_genre(
    id: i32,
    pool: &PgPool) -> sqlx::Result<String> {

    let genre = find_genre_by_id(id.clone(), pool).await?;
    let name = genre.name;

    let res = sqlx::query("DELETE FROM genres WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    tracing::info!("DB : Genre deleted : {}", &name);

    Ok(name)
}

pub async fn add_partition(
    title: String,
    person_name: String,
    genre_name: String,
    pool: &PgPool) -> sqlx::Result<Partition> {

    let mut person_id = 0;
    let mut genre_id = 0;

    if let Ok(person) = find_person_by_name(person_name, &pool).await {
        tracing::info!("from db::add_partition : personne : {:?}", person);
        person_id = person.id.unwrap();
    }
    else {
        return Err(Error::RowNotFound);
    };

    if let Ok(genre) = find_genre_by_name(genre_name, &pool).await {
        tracing::info!("from db::add_partition : genre : {:?}", genre);
        genre_id = genre.id.unwrap();
    } else {
        return Err(Error::RowNotFound);
    };


    let partition = sqlx::query(
        "INSERT INTO partitions (title, person_id, genre_id)
                VALUES ( $1, $2, $3 )
                RETURNING id, title, person_id, genre_id;"
    )
        .bind(&title)
        .bind(&person_id)
        .bind(&genre_id)
        .map(|row: PgRow| Partition {
            id: row.get(0),
            title: row.get(1),
            person_id: row.get(2),
            genre_id: row.get(3),
        })
        .fetch_one(pool)
        .await?;

    tracing::info!("DB : partition added : {:?}", &partition);

    Ok(partition)
}

pub async fn update_partition() {
    todo!()
}

pub async fn delete_partition(
    id: i32,
    pool: &PgPool) -> sqlx::Result<String> {

    let partition = find_partition_by_id(id.clone(), pool).await?;
    let name = partition.title;

    let res = sqlx::query("DELETE FROM partitions WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    tracing::info!("DB : Partition deleted : {}", &name);

    Ok(name)
}







// ************************************************************************************************
// Get vector with all occurences with same name or title
/*
pub async fn get_person_by_name(conn: &DBPool, person_full_name: String) -> QueryResult<Person> {
    conn.run(move |c|
        persons::table
            .filter(full_name.eq(person_full_name))
            .first(c)
    ).await
}

pub async fn get_genre_by_name(conn: &DBPool, genre_name: String) -> QueryResult<Genre> {
    conn.run(move |c|
        genres::table
            .filter(name.eq(genre_name))
            .first(c)
    ).await
}

pub async fn get_raw_partition_by_title(conn: &DBPool, partition_title: String) -> QueryResult<Partition> {
    conn.run(move |c|
        partitions::table
            .filter(title.eq(partition_title))
            .first(c)
    ).await
}

pub async fn get_partition_by_title(
    conn: &DBPool,
    partition_title: String,
) -> QueryResult<ShowPartition> {

    let raw_partition = get_raw_partition_by_title(conn, partition_title.clone()).await;
    match raw_partition {
        Ok(..) => {
            let data = conn.run(|c| {
                partitions::table
                    .inner_join(persons::table)
                    .inner_join(genres::table)
                    .select((
                        partitions::id,
                        partitions::title,
                        persons::full_name,
                        genres::name,
                    ))
                    .filter(partitions::title.eq(partition_title))
                    .first(c)
                    .expect("error")
            }).await;
            Ok(data)
        }
        Err(e) => Err(e)
    }

    let data = conn.run(|c| {
        partitions::table
            .inner_join(persons::table)
            .inner_join(genres::table)
            .select((
                partitions::id,
                partitions::title,
                persons::full_name,
                genres::name,
            ))
            .filter(partitions::title.eq(partition_title))
            .first(c)
            .expect("error")
    }).await;
    Ok(data)
}


pub async fn get_partition_by_author(
    conn: &DBPool,
    partition_author: String,
) -> QueryResult<Vec<ShowPartition>> {
    let pers = get_person_by_name(conn, partition_author).await.unwrap();

    let data = conn
        .run(move |c| {
            partitions::table
                .inner_join(persons::table)
                .inner_join(genres::table)
                .select((
                    partitions::id,
                    partitions::title,
                    persons::full_name,
                    genres::name,
                ))
                .filter(partitions::person_id.eq(pers.id.unwrap()))
                .load(c)
                .expect("error in finding partition by author")
        })
        .await;
    Ok(data)
}

pub async fn get_partition_by_genre(
    conn: &DBPool,
    partition_genre: String,
) -> QueryResult<Vec<ShowPartition>> {
    let genre = get_genre_by_name(conn, partition_genre).await.unwrap();

    let data = conn
        .run(move |c| {
            partitions::table
                .inner_join(persons::table)
                .inner_join(genres::table)
                .select((
                    partitions::id,
                    partitions::title,
                    persons::full_name,
                    genres::name,
                ))
                .filter(partitions::genre_id.eq(genre.id.unwrap()))
                .load(c)
                .expect("error in finding partition by genre")
        })
        .await;
    Ok(data)
}
*/
//*************************************************************************************************
// DELETE
/*
pub async fn delete_one_person(conn: &DBPool, person_id: i32) -> QueryResult<usize> {
    conn.run(move |c| diesel::delete(persons::table.find(person_id)).execute(c))
        .await
}

pub async fn delete_one_genre(conn: &DBPool, genre_id: i32) -> QueryResult<usize> {
    conn.run(move |c| diesel::delete(genres::table.find(genre_id)).execute(c))
        .await
}

pub async fn delete_one_partition(conn: &DBPool, partition_id: i32) -> QueryResult<usize> {
    conn.run(move |c| diesel::delete(partitions::table.find(partition_id)).execute(c))
        .await
}
*/
//*************************************************************************************************
// CREATE
/*
pub async fn create_person(conn: &DBPool, person: Person) -> QueryResult<Person> {
    conn.run(move |c| {
        diesel::insert_into(persons::table)
            .values(&person)
            .get_result(c)
    })
        .await
}

pub async fn create_genre(conn: &DBPool, genre: Genre) -> QueryResult<Genre> {
    conn.run(move |c| {
        diesel::insert_into(genres::table)
            .values(&genre)
            .get_result(c)
    })
        .await
}

pub async fn create_partition(
    conn: &DBPool,
    show_partition: ShowPartition,
) -> QueryResult<Partition> {
    let nom = show_partition.full_name.trim();

    let pers = get_person_by_name(conn, nom.to_string()).await?;
    println!("{:?}", pers);
    let g = get_genre_by_name(conn, show_partition.name).await?;
    println!("{:?}", g);
    let person_id = pers.id.unwrap();
    let genre_id = g.id.unwrap();

    let partition = Partition {
        id: None,
        person_id,
        title: show_partition.title,
        genre_id,
    };

    conn.run(move |c| {
        diesel::insert_into(partitions::table)
            .values(&partition)
            .get_result(c)
    })
        .await
}
*/
//******************************************************************************************
// UPDATE
/*
pub async fn update_person(pers_id: i32, person: Person, conn: &DBPool) -> QueryResult<Person> {
    conn.run(move |c| {
        diesel::update(persons::table.find(pers_id))
            .set(&person)
            .get_result(c)
    })
        .await
}

pub async fn update_genre(genre_id: i32, genre: Genre, conn: &DBPool) -> QueryResult<Genre> {
    conn.run(move |c| {
        diesel::update(genres::table.find(genre_id))
            .set(&genre)
            .get_result(c)
    })
        .await
}

pub async fn update_partition(
    part_id: i32,
    partition: Partition,
    conn: &DBPool,
) -> QueryResult<Partition> {
    conn.run(move |c| {
        diesel::update(partitions::table.find(part_id))
            .set(&partition)
            .get_result(c)
    })
        .await
}
*/
