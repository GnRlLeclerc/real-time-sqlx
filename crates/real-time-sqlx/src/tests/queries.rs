//! Serialized queries tests

use sqlx::FromRow;
use std::{fs, path::Path};

use crate::database::sqlite::fetch_sqlite_query;
use crate::queries::serialize::{QueryData, QueryTree};
use crate::tests::dummy::{dummy_sqlite_database, prepare_dummy_sqlite_database};

use super::dummy::Todo;
use super::utils::read_serialized_query;

#[tokio::test]
async fn test_deserialize_queries() {
    // Get the queries
    let queries_path = Path::new("src/tests/queries");

    for entry in fs::read_dir(queries_path).unwrap() {
        let entry = entry.unwrap();

        println!(
            "{:?}, {}",
            entry.path(),
            entry.file_name().to_str().unwrap()
        );

        let serialized_query = fs::read_to_string(entry.path()).unwrap();

        // Deserialize the query from json
        let query: serde_json::Value = serde_json::from_str(&serialized_query).unwrap();
        serde_json::from_value::<QueryTree>(query).expect(&format!(
            "Failed to deserialize query: {}",
            entry.file_name().into_string().unwrap()
        ));
    }
}

// ************************************************************************* //
//                     TESTING AGAINST SQLITE BACKEND                        //
// ************************************************************************* //

/// Test single row fetching
#[tokio::test]
async fn test_sqlite_single() {
    let pool = dummy_sqlite_database().await;
    prepare_dummy_sqlite_database(&pool).await;

    let query = read_serialized_query("01_single.json");
    let result = fetch_sqlite_query(&query, &pool).await;

    match result {
        QueryData::Single(row) => {
            let single_row = row.expect("Expected a single row");

            let data = Todo::from_row(&single_row).expect("Failed to convert single row");
            assert_eq!(data.id, 1);
            assert_eq!(data.title, "First todo");
            assert_eq!(data.content, "This is the first todo");
        }
        QueryData::Many(_) => panic!("Expected a single row"),
    }
}

/// Test many row fetching
#[tokio::test]
async fn test_sqlite_many() {
    let pool = dummy_sqlite_database().await;
    prepare_dummy_sqlite_database(&pool).await;

    let query = read_serialized_query("02_many.json");
    let result = fetch_sqlite_query(&query, &pool).await;

    match result {
        QueryData::Single(_) => {
            panic!("Expected many rows")
        }
        QueryData::Many(rows) => {
            assert_eq!(rows.len(), 3);

            let first_row = Todo::from_row(&rows[0]).expect("Failed to convert first row");
            assert_eq!(first_row.id, 1);
            assert_eq!(first_row.title, "First todo");
            assert_eq!(first_row.content, "This is the first todo");

            let second_row = Todo::from_row(&rows[1]).expect("Failed to convert second row");
            assert_eq!(second_row.id, 2);
            assert_eq!(second_row.title, "Second todo");
            assert_eq!(second_row.content, "This is the second todo");

            let third_row = Todo::from_row(&rows[2]).expect("Failed to convert third row");
            assert_eq!(third_row.id, 3);
            assert_eq!(third_row.title, "Third todo");
            assert_eq!(third_row.content, "This is the third todo");
        }
    }
}

/// Test single row fetching with a condition
#[tokio::test]
async fn test_sqlite_single_with_condition() {
    let pool = dummy_sqlite_database().await;
    prepare_dummy_sqlite_database(&pool).await;

    let query = read_serialized_query("03_single_with_condition.json");
    let result = fetch_sqlite_query(&query, &pool).await;

    match result {
        QueryData::Single(row) => {
            let single_row = row.expect("Expected a single row");

            let data = Todo::from_row(&single_row).expect("Failed to convert single row");
            assert_eq!(data.id, 2);
            assert_eq!(data.title, "Second todo");
            assert_eq!(data.content, "This is the second todo");
        }
        QueryData::Many(_) => panic!("Expected a single row"),
    }
}

/// Test many row fetching with a condition returning a single row
#[tokio::test]
async fn test_sqlite_many_with_condition() {
    let pool = dummy_sqlite_database().await;
    prepare_dummy_sqlite_database(&pool).await;

    let query = read_serialized_query("04_many_with_condition.json");
    let result = fetch_sqlite_query(&query, &pool).await;

    match result {
        QueryData::Single(_) => {
            panic!("Expected many rows")
        }
        QueryData::Many(rows) => {
            assert_eq!(rows.len(), 1);

            let data = Todo::from_row(&rows[0]).expect("Failed to convert first row");
            assert_eq!(data.id, 2);
            assert_eq!(data.title, "Second todo");
            assert_eq!(data.content, "This is the second todo");
        }
    }
}

/// Test fetching many rows with a nested OR condition
#[tokio::test]
async fn test_sqlite_nested_or() {
    let pool = dummy_sqlite_database().await;
    prepare_dummy_sqlite_database(&pool).await;

    let query = read_serialized_query("05_nested_or.json");
    let result = fetch_sqlite_query(&query, &pool).await;

    match result {
        QueryData::Single(_) => {
            panic!("Expected many rows")
        }
        QueryData::Many(rows) => {
            assert_eq!(rows.len(), 3);
        }
    }
}

/// Test single row fetching with no existing matching entry
#[tokio::test]
async fn test_sqlite_empty() {
    let pool = dummy_sqlite_database().await;
    prepare_dummy_sqlite_database(&pool).await;

    let query = read_serialized_query("06_empty.json");
    let result = fetch_sqlite_query(&query, &pool).await;

    match result {
        QueryData::Single(row) => {
            assert!(row.is_none());
        }
        QueryData::Many(_) => panic!("Expected a single row"),
    }
}
