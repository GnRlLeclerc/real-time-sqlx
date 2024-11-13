//! Particularized PostgreSQL implementations.

use sqlx::{postgres::PgRow, Executor, Postgres};

use crate::queries::serialize::{NativeType, QueryData, QueryTree, ReturnType};

use super::prepare_sqlx_query;

/// Fetch data using a serialized query tree from a PostgreSQL database
pub async fn fetch_postgres_query<'a, E>(query: &QueryTree, executor: E) -> QueryData<PgRow>
where
    E: Executor<'a, Database = Postgres>,
{
    // Prepare the query
    let (sql, values) = prepare_sqlx_query(&query);

    let mut sqlx_query = sqlx::query(&sql);

    // Bind the values
    for value in values {
        sqlx_query = match value {
            NativeType::Null => sqlx_query.bind(None::<String>),
            NativeType::Int(int) => sqlx_query.bind(int),
            NativeType::String(string) => sqlx_query.bind(string),
            NativeType::Bool(bool) => sqlx_query.bind(bool),
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

/// Helper function signature for serializing PostgreSQL rows to JSON
/// by mapping them to different data structs implementing `FromRow`
/// and `Serialize` depending on the table name.
pub type SerializeRowsMapped = fn(&QueryData<PgRow>, table: &str) -> serde_json::Value;
