//! Deserialize database queries from JSON

use serde::{Deserialize, Serialize};

/// Query final constraint value (ie "native" types)
/// Prevents recursive lists of values
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NativeType {
    Int(i32),
    String(String),
    Bool(bool),
    Null,
}

/// Query constraint value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConstraintValue {
    Final(NativeType),
    List(Vec<NativeType>),
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

/// Final serialized query tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTree {
    #[serde(rename = "return")]
    pub return_type: ReturnType,
    pub table: String,
    pub condition: Option<Condition>,
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
