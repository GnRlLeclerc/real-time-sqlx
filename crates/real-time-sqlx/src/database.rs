//! Query utilities and particularized database implementations
//! Some implementations need to be particularized because of trait generics hell.

use std::iter::repeat;

use serde::Serialize;
use sqlx::FromRow;

use crate::queries::serialize::{
    Condition, Constraint, ConstraintValue, NativeType, QueryData, QueryTree,
};

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "sqlite")]
pub mod sqlite;

/// Produce a prepared SQL string and a list of argument values for binding
/// from a deserialized query, and for use in a SQLx query
fn prepare_sqlx_query(query: &QueryTree) -> (String, Vec<NativeType>) {
    let mut string_query = "SELECT * FROM ".to_string();
    string_query.push_str(&query.table);

    if let Some(condition) = &query.condition {
        string_query.push_str(" WHERE ");
        let (placeholders, args) = condition.traverse();
        string_query.push_str(&placeholders);
        return (string_query, args);
    }

    (string_query, vec![])
}

/// Serialize SQL rows to json by mapping them to an intermediate data model structure
pub fn serialize_rows<T, R>(data: &QueryData<R>) -> serde_json::Value
where
    T: for<'r> FromRow<'r, R> + Serialize,
    R: sqlx::Row,
{
    match data {
        QueryData::Single(row) => match row {
            Some(row) => serde_json::json!(QueryData::Single(Some(T::from_row(row).unwrap()))),
            None => serde_json::json!(QueryData::Single(None::<T>)),
        },
        QueryData::Multiple(rows) => serde_json::json!(QueryData::Multiple(
            rows.iter()
                .map(|row| T::from_row(row).unwrap())
                .collect::<Vec<T>>()
        )),
    }
}

// ********************************************************************************************* //
//                                     Query Traversal Functions                                 //
// ********************************************************************************************* //

/// Trait to normalize the traversal of query constraints and conditions
trait Traversable {
    fn traverse(&self) -> (String, Vec<NativeType>);
}

impl Traversable for NativeType {
    /// Traverse a final constraint value
    fn traverse(&self) -> (String, Vec<NativeType>) {
        ("?".to_string(), vec![self.clone()])
    }
}

impl Traversable for ConstraintValue {
    /// Traverse a query constraint value
    fn traverse(&self) -> (String, Vec<NativeType>) {
        match self {
            ConstraintValue::List(list) => (
                format!(
                    "({})",
                    repeat("?".to_string())
                        .take(list.len())
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
                list.clone(),
            ),
            ConstraintValue::Final(value) => value.traverse(),
        }
    }
}

impl Traversable for Constraint {
    /// Traverse a query constraint
    fn traverse(&self) -> (String, Vec<NativeType>) {
        let (values_string_query, values) = self.value.traverse();

        (
            format!(
                "\"{}\" {} {}",
                self.column, self.operator, values_string_query
            ),
            values,
        )
    }
}

impl Traversable for Condition {
    /// Traverse a query condition
    fn traverse(&self) -> (String, Vec<NativeType>) {
        match self {
            Condition::Single { constraint } => constraint.traverse(),
            Condition::Or { conditions } => reduce_constraints_list(conditions, " OR "),
            Condition::And { conditions } => reduce_constraints_list(conditions, " AND "),
        }
    }
}

/// Create a list of string queries and constraint values vectors from a list of
/// conditions
fn reduce_constraints_list(conditions: &[Condition], sep: &str) -> (String, Vec<NativeType>) {
    let mut placeholder_strings: Vec<String> = vec![];
    let mut total_values: Vec<NativeType> = vec![];

    conditions.iter().for_each(|condition| {
        let (string_query, values) = condition.traverse();
        placeholder_strings.push(string_query);
        total_values.extend(values);
    });

    (format!("({})", placeholder_strings.join(sep)), total_values)
}
