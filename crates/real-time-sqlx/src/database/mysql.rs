//! Particularized MySQL implementations.

use sqlx::{
    mysql::{MySqlArguments, MySqlRow},
    query::Query,
    Executor, FromRow, MySql,
};

use crate::{
    operations::serialize::{GranularOperation, OperationNotification},
    queries::serialize::{FinalType, QueryData, QueryTree, ReturnType},
    utils::{
        delete_statement, insert_many_statement, insert_statement, ordered_keys, update_statement,
    },
};

use super::prepare_sqlx_query;

/// Bind a native value to a MySQL query
#[inline]
pub fn bind_mysql_value<'q>(
    query: Query<'q, MySql, MySqlArguments>,
    value: FinalType,
) -> Query<'q, MySql, MySqlArguments> {
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

/// Fetch data using a serialized query tree from a MySQL database
pub async fn fetch_mysql_query<'a, E>(query: &QueryTree, executor: E) -> QueryData<MySqlRow>
where
    E: Executor<'a, Database = MySql>,
{
    // Prepare the query
    let (sql, values) = prepare_sqlx_query(&query);

    let mut sqlx_query = sqlx::query(&sql);

    // Bind the values
    for value in values {
        sqlx_query = bind_mysql_value(sqlx_query, value);
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

/// Helper function signature for serializing MySQL rows to JSON
/// by mapping them to different data structs implementing `FromRow`
/// and `Serialize` depending on the table name.
pub type SerializeRowsMapped = fn(&QueryData<MySqlRow>, table: &str) -> serde_json::Value;

/// Perform a granular operation on a MySQL database.
/// Returns a notification to be sent to clients.
pub async fn granular_operation_mysql<'a, E, T>(
    operation: GranularOperation,
    executor: E,
) -> Option<OperationNotification<T>>
where
    E: Executor<'a, Database = MySql>,
    T: for<'r> FromRow<'r, MySqlRow>,
{
    match operation {
        GranularOperation::Create { table, mut data } => {
            // Fix the order of the keys for later iterations
            let keys = ordered_keys(&data);

            // Produce the SQL query string
            let string_query = insert_statement(&table, &keys);
            let mut sqlx_query = sqlx::query(&string_query);

            // Bind the values in the order of the keys
            for key in keys.iter() {
                // Consume the value and convert it to a NativeType for proper binding
                let value = data.remove(key).unwrap();
                let native_value = FinalType::try_from(value).unwrap();
                sqlx_query = bind_mysql_value(sqlx_query, native_value);
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
            let mut sqlx_query = sqlx::query(&string_query);

            // Bind all values in order of the keys
            for entry in data.iter_mut() {
                for key in keys.iter() {
                    // Consume the value and convert it to a NativeType for proper binding
                    let value = entry.remove(key).unwrap();
                    let native_value = FinalType::try_from(value).unwrap();
                    sqlx_query = bind_mysql_value(sqlx_query, native_value);
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
            let mut sqlx_query = sqlx::query(&string_query);

            // Bind the values in the order of the keys
            for key in keys.iter() {
                // Consume the value and convert it to a NativeType for proper binding
                let value = data.remove(key).unwrap();
                let native_value = FinalType::try_from(value).unwrap();
                sqlx_query = bind_mysql_value(sqlx_query, native_value);
            }

            // Bind the ID
            sqlx_query = bind_mysql_value(sqlx_query, id.clone());

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
            let mut sqlx_query = sqlx::query(&string_query);

            // Bind the ID
            sqlx_query = bind_mysql_value(sqlx_query, id.clone());

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
