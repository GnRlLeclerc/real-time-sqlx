//! Serialize and deserialize database operations from JSON

use serde::{Deserialize, Serialize};

use crate::queries::serialize::NativeType;

/// Generic JSON object type
pub type JsonObject = serde_json::Map<String, serde_json::Value>;

/// An incoming granular operation to be performed in the database
/// The data can be partial or complete, depending on the operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GranularOperation {
    #[serde(rename = "create")]
    Create { table: String, data: JsonObject },
    #[serde(rename = "create_many")]
    CreateMany {
        table: String,
        data: Vec<JsonObject>,
    },
    #[serde(rename = "update")]
    Update {
        table: String,
        id: NativeType,
        data: JsonObject,
    },
    #[serde(rename = "delete")]
    Delete { table: String, id: NativeType },
}

/// An outgoing operation notification to be sent to clients
/// The data sent back is always complete, hence the generic parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OperationNotification<T> {
    #[serde(rename = "create")]
    Create { table: String, data: T },
    #[serde(rename = "create_many")]
    CreateMany { table: String, data: Vec<T> },
    #[serde(rename = "update")]
    Update {
        table: String,
        id: NativeType,
        data: T,
    },
    #[serde(rename = "delete")]
    Delete { table: String, id: NativeType },
}
