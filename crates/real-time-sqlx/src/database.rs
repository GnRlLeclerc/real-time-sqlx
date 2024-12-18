//! Query utilities and particularized database implementations
//! Some implementations need to be particularized because of trait generics hell.

use serde::Serialize;
use sqlx::FromRow;

use crate::{
    queries::serialize::{
        Condition, Constraint, ConstraintValue, FinalType, OrderBy, PaginateOptions, QueryData,
        QueryTree,
    },
    utils::{placeholders, sanitize_identifier},
};

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "sqlite")]
pub mod sqlite;

/// Produce a prepared SQL string and a list of argument values for binding
/// from a deserialized query, and for use in a SQLx query
fn prepare_sqlx_query(query: &QueryTree) -> (String, Vec<FinalType>) {
    let mut string_query = "SELECT * FROM ".to_string();
    let mut values = vec![];
    string_query.push_str(&sanitize_identifier(&query.table));

    if let Some(condition) = &query.condition {
        string_query.push_str(" WHERE ");
        let (placeholders, args) = condition.traverse();
        string_query.push_str(&placeholders);
        values.extend(args);
    }

    if let Some(paginate) = &query.paginate {
        string_query.push_str(" ");
        let pagination = paginate.traverse();
        string_query.push_str(&pagination.0);
        values.extend(pagination.1);
    }

    (string_query, values)
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
        QueryData::Many(rows) => serde_json::json!(QueryData::Many(
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
    fn traverse(&self) -> (String, Vec<FinalType>);
}

impl Traversable for FinalType {
    /// Traverse a final constraint value
    fn traverse(&self) -> (String, Vec<FinalType>) {
        ("?".to_string(), vec![self.clone()])
    }
}

impl Traversable for ConstraintValue {
    /// Traverse a query constraint value
    fn traverse(&self) -> (String, Vec<FinalType>) {
        match self {
            ConstraintValue::List(list) => (placeholders(list.len()), list.clone()),
            ConstraintValue::Final(value) => value.traverse(),
        }
    }
}

impl Traversable for Constraint {
    /// Traverse a query constraint
    fn traverse(&self) -> (String, Vec<FinalType>) {
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
    fn traverse(&self) -> (String, Vec<FinalType>) {
        match self {
            Condition::Single { constraint } => constraint.traverse(),
            Condition::Or { conditions } => reduce_constraints_list(conditions, " OR "),
            Condition::And { conditions } => reduce_constraints_list(conditions, " AND "),
        }
    }
}

impl Traversable for PaginateOptions {
    /// Traverse a query pagination options
    fn traverse(&self) -> (String, Vec<FinalType>) {
        let mut query_string = "".to_string();
        let mut values: Vec<FinalType> = vec![];

        if let Some(order) = &self.order_by {
            query_string.push_str(
                match order {
                    OrderBy::Asc(col) => format!("ORDER BY {} ASC ", sanitize_identifier(col)),
                    OrderBy::Desc(col) => format!("ORDER BY {} DESC ", sanitize_identifier(col)),
                }
                .as_str(),
            );
        } else {
            // By default, if paginate options are present, order by ID descending
            query_string.push_str("ORDER BY id DESC ");
        }

        query_string.push_str("LIMIT ? ");
        values.push(FinalType::Number(self.per_page.into()));

        if let Some(offset) = self.offset {
            query_string.push_str("OFFSET ? ");
            values.push(FinalType::Number(offset.into()));
        }

        (query_string, values)
    }
}

/// Create a list of string queries and constraint values vectors from a list of
/// conditions
fn reduce_constraints_list(conditions: &[Condition], sep: &str) -> (String, Vec<FinalType>) {
    let mut placeholder_strings: Vec<String> = vec![];
    let mut total_values: Vec<FinalType> = vec![];

    conditions.iter().for_each(|condition| {
        let (string_query, values) = condition.traverse();
        placeholder_strings.push(string_query);
        total_values.extend(values);
    });

    (format!("({})", placeholder_strings.join(sep)), total_values)
}
