//! Test utilities

use std::path::Path;

use crate::{operations::serialize::GranularOperation, queries::serialize::QueryTree};

/// Read a serialized query into a QueryTree for execution
pub(crate) fn read_serialized_query(name: &str) -> QueryTree {
    // Load the file
    let path = Path::new("src/tests/queries").join(name);
    let serialized_query = std::fs::read_to_string(path).unwrap();

    // Deserialize the query from json
    let query: serde_json::Value = serde_json::from_str(&serialized_query).unwrap();
    serde_json::from_value(query).unwrap()
}

/// Read a serialized operation into a GranularOperation for execution
pub(crate) fn read_serialized_operation(name: &str) -> GranularOperation {
    // Load the file
    let path = Path::new("src/tests/operations").join(name);
    let serialized_operation = std::fs::read_to_string(path).unwrap();

    // Deserialize the operation from json
    let operation: serde_json::Value = serde_json::from_str(&serialized_operation).unwrap();
    serde_json::from_value(operation).unwrap()
}
