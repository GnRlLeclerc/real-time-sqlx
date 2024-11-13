//! Particularized SQLite implementations.

use sqlx::{sqlite::SqliteRow, Executor, Sqlite};

use crate::queries::serialize::{FinalConstraintValue, QueryData, QueryTree, ReturnType};

use super::prepare_sqlx_query;

/// Fetch data using a serialized query tree from a SQLite database
pub async fn fetch_sqlite_query<'a, E>(query: &QueryTree, executor: E) -> QueryData<SqliteRow>
where
    E: Executor<'a, Database = Sqlite>,
{
    // Prepare the query
    let (sql, values) = prepare_sqlx_query(&query);

    let mut sqlx_query = sqlx::query(&sql);

    // Bind the values
    for value in values {
        sqlx_query = match value {
            FinalConstraintValue::Null => sqlx_query.bind(None::<String>),
            FinalConstraintValue::Int(int) => sqlx_query.bind(int),
            FinalConstraintValue::String(string) => sqlx_query.bind(string),
            FinalConstraintValue::Bool(bool) => sqlx_query.bind(bool),
        };
    }

    // Fetch one or many rows depending on the query
    match query.return_type {
        ReturnType::Single => {
            let row = sqlx_query.fetch_one(executor).await.ok();
            return QueryData::Single(row);
        }
        ReturnType::Multiple => {
            let rows = sqlx_query.fetch_all(executor).await.unwrap();
            return QueryData::Multiple(rows);
        }
    }
}

/// Helper function signature for serializing SQLite rows to JSON
/// by mapping them to different data structs implementing `FromRow`
/// and `Serialize` depending on the table name.
pub type SerializeRowsMapped = fn(&QueryData<SqliteRow>, table: &str) -> serde_json::Value;
