//! Serialized operations tests

use std::{fs, path::Path};

use crate::database::sqlite::granular_operation_sqlite;
use crate::operations::serialize::{GranularOperation, OperationNotification};
use crate::tests::dummy::{dummy_sqlite_database, prepare_dummy_sqlite_database};

use super::dummy::Todo;
use super::utils::read_serialized_operation;

#[tokio::test]
async fn test_deserialize_operations() {
    // Get the operations
    let operations_path = Path::new("src/tests/operations");

    for entry in fs::read_dir(operations_path).unwrap() {
        let entry = entry.unwrap();

        let serialized_operation = fs::read_to_string(entry.path()).unwrap();

        // Deserialize the query from json
        let query: serde_json::Value = serde_json::from_str(&serialized_operation).unwrap();
        serde_json::from_value::<GranularOperation>(query).expect(&format!(
            "Failed to deserialize operation: {}",
            entry.file_name().into_string().unwrap()
        ));
    }
}

// ************************************************************************* //
//                     TESTING AGAINST SQLITE BACKEND                        //
// ************************************************************************* //

/// Test single row creation
#[tokio::test]
async fn test_sqlite_create() {
    let pool = dummy_sqlite_database().await;
    prepare_dummy_sqlite_database(&pool).await;

    let operation = read_serialized_operation("01_create.json");
    let result = granular_operation_sqlite(operation, &pool).await;

    assert!(result.is_some());
    let result: OperationNotification<Todo> = result.unwrap();

    match result {
        OperationNotification::Create { table: _, data } => {
            assert_eq!(data.id, 4);
            assert_eq!(data.title, "Fourth todo");
            assert_eq!(data.content, "This is the fourth todo");
        }
        _ => panic!("Expected a create operation"),
    }
}

/// Test multiple row creation
#[tokio::test]
async fn test_sqlite_create_many() {
    let pool = dummy_sqlite_database().await;
    prepare_dummy_sqlite_database(&pool).await;

    let operation = read_serialized_operation("02_create_many.json");
    let result = granular_operation_sqlite(operation, &pool).await;

    assert!(result.is_some());
    let result: OperationNotification<Todo> = result.unwrap();

    match result {
        OperationNotification::CreateMany { table: _, data } => {
            assert_eq!(data.len(), 2);

            let first_data = &data[0];
            assert_eq!(first_data.id, 4);
            assert_eq!(first_data.title, "Fourth todo");
            assert_eq!(first_data.content, "This is the fourth todo");

            let second_data = &data[1];
            assert_eq!(second_data.id, 5);
            assert_eq!(second_data.title, "Fifth todo");
            assert_eq!(second_data.content, "This is the fifth todo");
        }
        _ => panic!("Expected a create many operation"),
    }
}

/// Test single row update
#[tokio::test]
async fn test_sqlite_update() {
    let pool = dummy_sqlite_database().await;
    prepare_dummy_sqlite_database(&pool).await;

    let operation = read_serialized_operation("03_update.json");
    let result = granular_operation_sqlite(operation, &pool).await;

    assert!(result.is_some());
    let result: OperationNotification<Todo> = result.unwrap();

    match result {
        OperationNotification::Update {
            table: _,
            id: _,
            data,
        } => {
            assert_eq!(data.id, 3);
            assert_eq!(data.title, "Updated todo");
            assert_eq!(data.content, "This todo was updated");
        }
        _ => panic!("Expected an update operation"),
    }
}

/// Test single row deletion
#[tokio::test]
async fn test_sqlite_delete() {
    let pool = dummy_sqlite_database().await;
    prepare_dummy_sqlite_database(&pool).await;

    let operation = read_serialized_operation("04_delete.json");
    let result = granular_operation_sqlite(operation, &pool).await;

    assert!(result.is_some());
    let result: OperationNotification<Todo> = result.unwrap();

    match result {
        OperationNotification::Delete { .. } => {
            // Nothing to check here. The operation is not None, meaning 1 row ws affected
        }
        _ => panic!("Expected a delete operation"),
    }
}
