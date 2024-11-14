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

// TODO : rest
