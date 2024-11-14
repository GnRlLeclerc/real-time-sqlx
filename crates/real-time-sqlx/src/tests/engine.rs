//! Tests for the in-memory simple query engine.
//! It must give the same matches as the SQL query engine in the tests.

// ************************************************************************* //
//                     TESTING AGAINST SQLITE BACKEND                        //
// ************************************************************************* //

use sqlx::FromRow;

use crate::{
    database::sqlite::fetch_sqlite_query,
    operations::serialize::object_from_value,
    queries::{serialize::QueryTree, Checkable},
};

use super::{
    dummy::{dummy_sqlite_database, prepare_dummy_sqlite_database, Todo},
    utils::read_serialized_query,
};

/// Return the list of all todos in the default dummy database
/// for comparison with the SQL query engine
fn todos() -> Vec<Todo> {
    vec![
        Todo {
            id: 1,
            title: "First todo".to_string(),
            content: "This is the first todo".to_string(),
        },
        Todo {
            id: 2,
            title: "Second todo".to_string(),
            content: "This is the second todo".to_string(),
        },
        Todo {
            id: 3,
            title: "Third todo".to_string(),
            content: "This is the third todo".to_string(),
        },
    ]
}

/// Returns a vector of the todos that match the input query
fn filter_todos(query: &QueryTree) -> Vec<Todo> {
    todos()
        .into_iter()
        .filter(|t| query.check(&object_from_value(serde_json::to_value(t).unwrap()).unwrap()))
        .collect()
}

/// Test single row fetching
#[tokio::test]
async fn test_engine_single() {
    let query = read_serialized_query("01_single.json");
    let engine_todos = filter_todos(&query);

    // NOTE: the engine matches all 3 Todos, because the query actually does.
    // The real-time query engine does not account for "single" return type.
    // The frontend will have to handle this degenerate case where one random
    // row is fetched from the database without conditions strict enough for some reason.
    assert_eq!(engine_todos.len(), 3);
}

/// Test many row fetching
#[tokio::test]
async fn test_engine_many() {
    let pool = dummy_sqlite_database().await;
    prepare_dummy_sqlite_database(&pool).await;

    let query = read_serialized_query("02_many.json");
    let result = fetch_sqlite_query(&query, &pool).await;
    let all_rows = result.unwrap_many();

    let engine_todos = filter_todos(&query);

    assert_eq!(engine_todos.len(), all_rows.len());
}

/// Test single row fetching with a condition
#[tokio::test]
async fn test_engine_single_with_condition() {
    let pool = dummy_sqlite_database().await;
    prepare_dummy_sqlite_database(&pool).await;

    let query = read_serialized_query("03_single_with_condition.json");
    let result = fetch_sqlite_query(&query, &pool).await;
    let single_row = Todo::from_row(&result.unwrap_single()).unwrap();

    let engine_todos = filter_todos(&query);

    assert_eq!(engine_todos.len(), 1);
    assert_eq!(engine_todos[0], single_row);
}

/// Test many row fetching with a condition returning a single row
#[tokio::test]
async fn test_engine_many_with_condition() {
    let pool = dummy_sqlite_database().await;
    prepare_dummy_sqlite_database(&pool).await;

    let query = read_serialized_query("04_many_with_condition.json");
    let result = fetch_sqlite_query(&query, &pool).await;
    let single_row = Todo::from_row(&result.unwrap_many()[0]).unwrap();

    let engine_todos = filter_todos(&query);

    assert_eq!(engine_todos.len(), 1);
    assert_eq!(engine_todos[0], single_row);
}

/// Test fetching many rows with a nested OR condition
#[tokio::test]
async fn test_engine_nested_or() {
    let pool = dummy_sqlite_database().await;
    prepare_dummy_sqlite_database(&pool).await;

    let query = read_serialized_query("05_nested_or.json");
    let result = fetch_sqlite_query(&query, &pool).await;
    let all_rows = result.unwrap_many();

    let engine_todos = filter_todos(&query);

    assert_eq!(engine_todos.len(), all_rows.len());
}

/// Test single row fetching with no existing matching entry
#[tokio::test]
async fn test_engine_empty() {
    let pool = dummy_sqlite_database().await;
    prepare_dummy_sqlite_database(&pool).await;

    let query = read_serialized_query("06_empty.json");
    let result = fetch_sqlite_query(&query, &pool).await;
    let single_row = result.unwrap_optional_single();

    let engine_todos = filter_todos(&query);

    assert!(single_row.is_none());
    assert_eq!(engine_todos.len(), 0);
}

/// Test `IN` operations with arrays
#[tokio::test]
async fn test_engine_in() {
    let pool = dummy_sqlite_database().await;
    prepare_dummy_sqlite_database(&pool).await;

    let query = read_serialized_query("07_in.json");
    let result = fetch_sqlite_query(&query, &pool).await;
    let all_rows = result
        .unwrap_many()
        .into_iter()
        .map(|r| Todo::from_row(&r).unwrap())
        .collect::<Vec<Todo>>();

    let engine_todos = filter_todos(&query);

    assert_eq!(engine_todos, all_rows);
}
