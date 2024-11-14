//! Serialize and deserialize database operations from JSON

use serde::{Deserialize, Serialize};

use crate::queries::serialize::FinalType;

/// Generic JSON object type
pub type JsonObject = serde_json::Map<String, serde_json::Value>;

/// Entities related to a specific table
pub trait Tabled {
    fn get_table(&self) -> &str;
}

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
        id: FinalType,
        data: JsonObject,
    },
    #[serde(rename = "delete")]
    Delete { table: String, id: FinalType },
}

impl Tabled for GranularOperation {
    /// Helper method to get the table name from the operation
    fn get_table(&self) -> &str {
        match self {
            GranularOperation::Create { table, .. } => table,
            GranularOperation::CreateMany { table, .. } => table,
            GranularOperation::Update { table, .. } => table,
            GranularOperation::Delete { table, .. } => table,
        }
    }
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
        id: FinalType,
        data: T,
    },
    #[serde(rename = "delete")]
    Delete { table: String, id: FinalType },
}

impl<T> Tabled for OperationNotification<T> {
    /// Helper method to get the table name from the operation
    fn get_table(&self) -> &str {
        match self {
            OperationNotification::Create { table, .. } => table,
            OperationNotification::CreateMany { table, .. } => table,
            OperationNotification::Update { table, .. } => table,
            OperationNotification::Delete { table, .. } => table,
        }
    }
}
