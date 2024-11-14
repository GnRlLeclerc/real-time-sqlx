//! Serialized queries tests

use std::{fs, path::Path};

use crate::queries::serialize::QueryTree;

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

// TODO : execute all queries against sqlite
