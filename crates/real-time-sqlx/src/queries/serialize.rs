//! Deserialize database queries from JSON

use serde::{Deserialize, Serialize};
use serde_json::Number;

use crate::error::DeserializeError;

/// Query final constraint value (ie "native" types)
/// Prevents recursive lists of values
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FinalType {
    Number(Number),
    String(String),
    Bool(bool),
    Null,
}

/// For binding values to queries, JSON values must be converted to native types
/// in order to avoid cases such as double quotes enclosed strings.
impl TryFrom<serde_json::Value> for FinalType {
    type Error = DeserializeError;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        match value {
            serde_json::Value::Number(n) => Ok(FinalType::Number(n)),
            serde_json::Value::String(s) => Ok(FinalType::String(s)),
            serde_json::Value::Bool(b) => Ok(FinalType::Bool(b)),
            serde_json::Value::Null => Ok(FinalType::Null),
            value => Err(DeserializeError::IncompatibleValue(value)),
        }
    }
}

/// Query constraint value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConstraintValue {
    Final(FinalType),
    List(Vec<FinalType>),
}

/// Constraint operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operator {
    #[serde(rename = "=")]
    Equal,
    #[serde(rename = "<")]
    LessThan,
    #[serde(rename = ">")]
    GreaterThan,
    #[serde(rename = "<=")]
    LessThanOrEqual,
    #[serde(rename = ">=")]
    GreaterThanOrEqual,
    #[serde(rename = "!=")]
    NotEqual,
    #[serde(rename = "in")]
    In,
    #[serde(rename = "like")]
    Like,
    #[serde(rename = "ilike")]
    ILike,
}

/// Query constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub column: String,
    pub operator: Operator,
    pub value: ConstraintValue,
}

/// Query condition (contains constraints)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Condition {
    #[serde(rename = "and")]
    And { conditions: Vec<Condition> },
    #[serde(rename = "or")]
    Or { conditions: Vec<Condition> },
    #[serde(rename = "single")]
    Single { constraint: Constraint },
}

/// Query return type (single row vs multiple rows)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReturnType {
    #[serde(rename = "single")]
    Single,
    #[serde(rename = "many")]
    Many,
}

/// Column and order for sorting
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "order", content = "column")]
pub enum OrderBy {
    #[serde(rename = "asc")]
    Asc(String),
    #[serde(rename = "desc")]
    Desc(String),
}

/// Pagination options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginateOptions {
    #[serde(rename = "perPage")]
    pub per_page: u64,
    pub offset: Option<u64>,
    #[serde(rename = "orderBy")]
    pub order_by: Option<OrderBy>,
}

/// Final serialized query tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTree {
    #[serde(rename = "return")]
    pub return_type: ReturnType,
    pub table: String,
    pub condition: Option<Condition>,
    pub paginate: Option<PaginateOptions>,
}

/// Returned query data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum QueryData<D> {
    #[serde(rename = "single")]
    Single(Option<D>),
    #[serde(rename = "many")]
    Many(Vec<D>),
}

/// Helper implementations for unwrapping query data
impl<D> QueryData<D> {
    pub fn unwrap_single(self) -> D {
        match self {
            QueryData::Single(Some(data)) => data,
            QueryData::Single(None) => panic!("No data found"),
            QueryData::Many(_) => panic!("Expected single row, found multiple rows"),
        }
    }

    pub fn unwrap_optional_single(self) -> Option<D> {
        match self {
            QueryData::Single(data) => data,
            QueryData::Many(_) => panic!("Expected single row, found multiple rows"),
        }
    }

    pub fn unwrap_many(self) -> Vec<D> {
        match self {
            QueryData::Single(_) => panic!("Expected multiple rows, found single row"),
            QueryData::Many(data) => data,
        }
    }
}
