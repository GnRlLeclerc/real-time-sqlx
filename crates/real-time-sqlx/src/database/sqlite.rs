//! Particularized SQLite implementations.

use sqlx::{
    query::Query,
    sqlite::{SqliteArguments, SqliteRow},
    Column, Executor, FromRow, Row, Sqlite, TypeInfo,
};

use crate::{
    operations::serialize::{GranularOperation, OperationNotification},
    queries::serialize::{FinalType, QueryData, QueryTree, ReturnType},
    utils::{
        delete_statement, insert_many_statement, insert_statement, ordered_keys,
        to_numbered_placeholders, update_statement,
    },
};

use super::prepare_sqlx_query;

/// Bind a native value to a Sqlite query
#[inline]
pub fn bind_sqlite_value<'q>(
    query: Query<'q, Sqlite, SqliteArguments<'q>>,
    value: FinalType,
) -> Query<'q, Sqlite, SqliteArguments<'q>> {
    match value {
        FinalType::Null => query.bind(None::<String>),
        FinalType::Number(number) => {
            if number.is_f64() {
                query.bind(number.as_f64().unwrap())
            } else {
                query.bind(number.as_i64().unwrap())
            }
        }
        FinalType::String(string) => query.bind(string),
        FinalType::Bool(bool) => query.bind(bool),
    }
}

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
        sqlx_query = bind_sqlite_value(sqlx_query, value);
    }

    // Fetch one or many rows depending on the query
    match query.return_type {
        ReturnType::Single => {
            let row = sqlx_query.fetch_optional(executor).await.unwrap();
            return QueryData::Single(row);
        }
        ReturnType::Many => {
            let rows = sqlx_query.fetch_all(executor).await.unwrap();
            return QueryData::Many(rows);
        }
    }
}

/// Convert a SQLite row to a JSON object
pub fn sqlite_row_to_json(row: &SqliteRow) -> serde_json::Value {
    let mut json_map = serde_json::Map::new();

    for column in row.columns() {
        let column_name = column.name();
        let column_type = column.type_info().name();

        // Dynamically match the type and insert it into the JSON map
        let value = match column_type {
            "INTEGER" => row
                .try_get::<i64, _>(column_name)
                .ok()
                .map(serde_json::Value::from),
            "REAL" | "NUMERIC" => row
                .try_get::<f64, _>(column_name)
                .ok()
                .map(serde_json::Value::from),
            "BOOLEAN" => row
                .try_get::<bool, _>(column_name)
                .ok()
                .map(serde_json::Value::from),
            "TEXT" | "DATE" | "TIME" | "DATETIME" => row
                .try_get::<String, _>(column_name)
                .ok()
                .map(serde_json::Value::from),
            "NULL" => Some(serde_json::Value::Null),
            "BLOB" => None, // Skip BLOB columns
            _ => None,      // Handle other types as needed
        };

        // Add to JSON map if value is present
        if let Some(v) = value {
            json_map.insert(column_name.to_string(), v);
        } else {
            json_map.insert(column_name.to_string(), serde_json::Value::Null);
        }
    }

    serde_json::Value::Object(json_map)
}

/// Convert a vector of SQLite rows to a JSON array
pub fn sqlite_rows_to_json(rows: &[SqliteRow]) -> serde_json::Value {
    let mut json_array = Vec::new();

    for row in rows {
        json_array.push(sqlite_row_to_json(row));
    }

    serde_json::Value::Array(json_array)
}

/// Helper function signature for serializing SQLite rows to JSON
/// by mapping them to different data structs implementing `FromRow`
/// and `Serialize` depending on the table name.
pub type SerializeRowsMapped = fn(&QueryData<SqliteRow>, table: &str) -> serde_json::Value;

/// Perform a granular operation on a SQLite database.
/// Returns a notification to be sent to clients.
pub async fn granular_operation_sqlite<'a, E, T>(
    operation: GranularOperation,
    executor: E,
) -> Option<OperationNotification<T>>
where
    E: Executor<'a, Database = Sqlite>,
    T: for<'r> FromRow<'r, SqliteRow>,
{
    match operation {
        GranularOperation::Create { table, mut data } => {
            // Fix the order of the keys for later iterations
            let keys = ordered_keys(&data);

            // Produce the SQL query string
            let string_query = insert_statement(&table, &keys);
            let numbered_query = to_numbered_placeholders(&string_query);

            let mut sqlx_query = sqlx::query(&numbered_query);

            // Bind the values in the order of the keys
            for key in keys.iter() {
                // Consume the value and convert it to a NativeType for proper binding
                let value = data.remove(key).unwrap();
                let native_value = FinalType::try_from(value).unwrap();
                sqlx_query = bind_sqlite_value(sqlx_query, native_value);
            }

            let result = sqlx_query.fetch_one(executor).await.unwrap();
            let data = T::from_row(&result).unwrap();

            // Produce the creation notification
            Some(OperationNotification::Create {
                table: table.to_string(),
                data,
            })
        }
        GranularOperation::CreateMany { table, mut data } => {
            // Fix the order of the keys for later iterations
            let keys = ordered_keys(&data[0]);

            // Produce the SQL query string
            let string_query = insert_many_statement(&table, &keys, data.len());
            let numbered_query = to_numbered_placeholders(&string_query);

            let mut sqlx_query = sqlx::query(&numbered_query);

            // Bind all values in order of the keys
            for entry in data.iter_mut() {
                for key in keys.iter() {
                    // Consume the value and convert it to a NativeType for proper binding
                    let value = entry.remove(key).unwrap();
                    let native_value = FinalType::try_from(value).unwrap();
                    sqlx_query = bind_sqlite_value(sqlx_query, native_value);
                }
            }

            let results = sqlx_query.fetch_all(executor).await.unwrap();
            let data: Vec<T> = results
                .into_iter()
                .map(|row| T::from_row(&row).unwrap())
                .collect();

            // Produce the operation notification
            Some(OperationNotification::CreateMany {
                table: table.to_string(),
                data,
            })
        }
        GranularOperation::Update {
            table,
            id,
            mut data,
        } => {
            // Fix the order of the keys for later iterations
            let keys = ordered_keys(&data);

            // Produce the SQL query string
            let string_query = update_statement(&table, &keys);
            let numbered_query = to_numbered_placeholders(&string_query);

            let mut sqlx_query = sqlx::query(&numbered_query);

            // Bind the values in the order of the keys
            for key in keys.iter() {
                // Consume the value and convert it to a NativeType for proper binding
                let value = data.remove(key).unwrap();
                let native_value = FinalType::try_from(value).unwrap();
                sqlx_query = bind_sqlite_value(sqlx_query, native_value);
            }

            // Bind the ID
            sqlx_query = bind_sqlite_value(sqlx_query, id.clone());

            let result = sqlx_query.fetch_optional(executor).await.unwrap();

            if result.is_none() {
                return None;
            }

            let data = T::from_row(&result.unwrap()).unwrap();

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
            sqlx_query = bind_sqlite_value(sqlx_query, id.clone());

            let result = sqlx_query.fetch_optional(executor).await.unwrap();

            if result.is_none() {
                return None;
            }

            let data = T::from_row(&result.unwrap()).unwrap();

            Some(OperationNotification::Delete {
                table: table.to_string(),
                id: id.clone(),
                data,
            })
        }
    }
}
