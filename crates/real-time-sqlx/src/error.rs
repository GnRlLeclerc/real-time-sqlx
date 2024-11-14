//! Custom errors

use thiserror::Error;

/// Deserialization errors
#[derive(Error, Debug)]
pub enum DeserializeError {
    #[error("JSON Value could not be deserialized to FinalType")]
    IncompatibleValue(serde_json::Value),
    #[error("JSON Value could not be coerced to a Map")]
    IncompatibleMap(serde_json::Value),
}
