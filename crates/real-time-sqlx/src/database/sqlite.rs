//! Particularized SQLite implementations.

use sqlx::{sqlite::SqliteRow, Executor, Sqlite};

use crate::{
    operations::serialize::{GranularOperation, OperationNotification},
    queries::serialize::{NativeType, QueryData, QueryTree, ReturnType},
    utils::{delete_statement, insert_statement, to_numbered_placeholders, update_statement},
};

use super::prepare_sqlx_query;

/// Fetch data using a serialized query tree from a SQLite database
pub async fn fetch_sqlite_query<'a, E>(query: &QueryTree, executor: E) -> QueryData<SqliteRow>
where
    E: Executor<'a, Database = Sqlite>,
{
    // Prepare the query
    let (sql, values) = prepare_sqlx_query(&query);
    let with_placeholders = to_numbered_placeholders(&sql);
    let mut sqlx_query = sqlx::query(&with_placeholders);

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
            let row = sqlx_query.fetch_optional(executor).await.unwrap();
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

/// Perform a granular operation on a SQLite database.
/// Returns a notification to be sent to clients.
pub async fn granular_operation_sqlite<'a, E, T>(
    operation: &GranularOperation,
    executor: E,
) -> Option<OperationNotification<T>>
where
    E: Executor<'a, Database = Sqlite>,
    T: From<SqliteRow>,
{
    match operation {
        GranularOperation::Create { table, data } => {
            // Extract the keys and values from the JSON object
            let keys: Vec<&String> = data.keys().collect(); // Get the keys in a specific order

            // Produce the SQL query string
            let string_query = insert_statement(&table, &keys);
            let numbered_query = to_numbered_placeholders(&string_query);

            let mut sqlx_query = sqlx::query(&numbered_query);

            // Bind the values in the order of the keys
            for key in keys.iter() {
                let value = data.get(*key).unwrap();
                sqlx_query = sqlx_query.bind(value);
            }

            let result = sqlx_query.fetch_one(executor).await.unwrap();
            let data: T = result.into();

            // Produce the creation notification
            Some(OperationNotification::Create {
                table: table.to_string(),
                data,
            })
        }
        GranularOperation::CreateMany { table, data } => {
            // Extract the keys and values from the JSON object
            let keys: Vec<&String> = data[0].keys().collect(); // Get the keys in a specific order

            // Produce the SQL query string
            let string_query = insert_statement(&table, &keys);
            let numbered_query = to_numbered_placeholders(&string_query);

            let mut sqlx_query = sqlx::query(&numbered_query);

            // Bind all values in order of the keys
            for entry in data.iter() {
                for key in keys.iter() {
                    let value = entry.get(*key).unwrap();
                    sqlx_query = sqlx_query.bind(value);
                }
            }

            let results = sqlx_query.fetch_all(executor).await.unwrap();
            let data: Vec<T> = results.into_iter().map(|row| row.into()).collect();

            // Produce the operation notification
            Some(OperationNotification::CreateMany {
                table: table.to_string(),
                data,
            })
        }
        GranularOperation::Update { table, id, data } => {
            // Extract the keys and values from the JSON object
            let keys: Vec<&String> = data.keys().collect(); // Get the keys in a specific order

            // Produce the SQL query string
            let string_query = update_statement(&table, &keys);
            let numbered_query = to_numbered_placeholders(&string_query);

            let mut sqlx_query = sqlx::query(&numbered_query);

            // Bind the values in the order of the keys
            for key in keys.iter() {
                let value = data.get(*key).unwrap();
                sqlx_query = sqlx_query.bind(value);
            }

            // Bind the ID
            sqlx_query = match id {
                NativeType::Null => sqlx_query.bind(None::<String>),
                NativeType::Int(int) => sqlx_query.bind(int),
                NativeType::String(string) => sqlx_query.bind(string),
                NativeType::Bool(bool) => sqlx_query.bind(bool),
            };

            let result = sqlx_query.fetch_optional(executor).await.unwrap();

            if result.is_none() {
                return None;
            }

            let data: T = result.unwrap().into();

            // Produce the creation notification
            Some(OperationNotification::Update {
                table: table.to_string(),
                id: id.clone(),
                data,
            })
        }
        GranularOperation::Delete { table, id } => {
            let string_query = delete_statement(&table);
            let numbered_query = to_numbered_placeholders(&string_query);

            let mut sqlx_query = sqlx::query(&numbered_query);

            // Bind the ID
            sqlx_query = match id {
                NativeType::Null => sqlx_query.bind(None::<String>),
                NativeType::Int(int) => sqlx_query.bind(int),
                NativeType::String(string) => sqlx_query.bind(string),
                NativeType::Bool(bool) => sqlx_query.bind(bool),
            };

            let result = sqlx_query.execute(executor).await.unwrap().rows_affected();

            if result == 0 {
                return None;
            }

            Some(OperationNotification::Delete {
                table: table.to_string(),
                id: id.clone(),
            })
        }
    }
}
