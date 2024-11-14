//! Dummy data for testing

use std::fs;

use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, MySql, Pool, Postgres, Sqlite};

/// A dummy struct for testing purposes
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Todo {
    pub id: i32,
    pub title: String,
    pub content: String,
}

#[cfg(feature = "sqlite")]
/// Create an in-memory Sqlite database and return a pool connection
pub async fn dummy_sqlite_database() -> Pool<Sqlite> {
    sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create an in-memory sqlite database")
}

#[cfg(feature = "sqlite")]
/// Create and seed a Sqlite database from a pool connection
pub async fn prepare_dummy_sqlite_database(pool: &Pool<Sqlite>) {
    let mut tx = pool.begin().await.unwrap();

    let create_stmt = fs::read_to_string("src/tests/sql/01_create.sql").unwrap();
    let query = sqlx::query(&create_stmt);

    query
        .execute(&mut *tx)
        .await
        .expect("Failed to create a dummy database");

    let insert_stmt = fs::read_to_string("src/tests/sql/02_insert.sql").unwrap();
    let query = sqlx::query(&insert_stmt);

    query
        .execute(&mut *tx)
        .await
        .expect("Failed to insert dummy data");

    tx.commit()
        .await
        .expect("Failed to prepare a dummy database");
}

#[cfg(feature = "mysql")]
/// Create and seed a MySQL database from a pool connection
pub async fn prepare_dummy_mysql_database(pool: &Pool<MySql>) {
    let mut tx = pool.begin().await.unwrap();

    let create_stmt = fs::read_to_string("src/tests/sql/01_create.sql").unwrap();
    let query = sqlx::query(&create_stmt);

    query
        .execute(&mut *tx)
        .await
        .expect("Failed to create a dummy database");

    let insert_stmt = fs::read_to_string("src/tests/sql/02_insert.sql").unwrap();
    let query = sqlx::query(&insert_stmt);

    query
        .execute(&mut *tx)
        .await
        .expect("Failed to insert dummy data");

    tx.commit()
        .await
        .expect("Failed to prepare a dummy database");
}

#[cfg(feature = "postgres")]
/// Create and seed a Postgres database from a pool connection
pub async fn prepare_dummy_postgres_database(pool: &Pool<Postgres>) {
    let mut tx = pool.begin().await.unwrap();

    let create_stmt = fs::read_to_string("src/tests/sql/01_create.sql").unwrap();
    let query = sqlx::query(&create_stmt);

    query
        .execute(&mut *tx)
        .await
        .expect("Failed to create a dummy database");

    let insert_stmt = fs::read_to_string("src/tests/sql/02_insert.sql").unwrap();
    let query = sqlx::query(&insert_stmt);

    query
        .execute(&mut *tx)
        .await
        .expect("Failed to insert dummy data");

    tx.commit()
        .await
        .expect("Failed to prepare a dummy database");
}
