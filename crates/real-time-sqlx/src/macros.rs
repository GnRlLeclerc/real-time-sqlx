//! Helper macros to automatically generate static dispatcher code between models.

pub extern crate paste;

/// Macro that generates the static rows serialization dispatcher function,
/// that given sqlite rows, serializes them to the appropriate model based on the table name.
///
/// Example:
/// ```ignore
/// // Generate the function
/// serialize_rows_static!(sqlite, ("todos", Todo), ("users", User));
///
/// // Use it to serialize `QueryData<Row>` to JSON, with a table name.
/// let serialized: serde_json::Value = serialize_rows_static(&rows, "todos");
/// ```
#[macro_export]
macro_rules! serialize_rows_static {
    ($db_type:ident, $(($table_name:literal, $struct:ty)),+ $(,)?) => {
        fn serialize_rows_static(data: &$crate::queries::serialize::QueryData<$crate::database_row!($db_type)>, table: &str) -> serde_json::Value {
            match table {
                $(
                    $table_name => $crate::database::serialize_rows::<$struct, $crate::database_row!($db_type)>(data),
                )+
                _ => panic!("Table not found"),
            }
        }
    };
}

/// Macro that generates a static operation executor and serializer function,
/// that given a granular operation, executes it, parses the result into the data structure
/// corresponding to the table name, and serializes it to JSON. This is useful for simple operation
/// processing, without real-time updates.
///
/// Example:
/// ```ignore
/// // Generate the function`
/// granular_operations!(sqlite, ("todos", Todo), ("users", User));
///
/// // Use it to execute a granular operation and serialize the result to JSON.
/// let serialized: serde_json::Value = granular_operation_static(operation, &pool).await;
/// ```
#[macro_export]
macro_rules! granular_operations {
    ($db_type:ident, $(($table_name:literal, $struct:ty)),+ $(,)?) => {
        async fn granular_operation_static(
            operation: $crate::operations::serialize::GranularOperation,
            pool: &$crate::database_pool!($db_type),
        ) -> serde_json::Value {
            match operation.get_table() {
                $(
                    $table_name => {
                        // Dynamically invoke the correct database function based on $db_type
                        let result: Option<$crate::operations::serialize::OperationNotification<$struct>> =
                            $crate::granular_operation_fn!($db_type)(operation, pool).await;
                        serde_json::to_value(result).unwrap()
                    }
                )+
                _ => panic!("Table not found"),
            }
        }
    };
}

// ************************************************************************* //
//        HELPER MACROS - RESOLVE DATABASE SPECIFIC FUNCTIONS AND TYPES      //
// ************************************************************************* //

/// Returns the appropriate database pool type based on the database type.
#[macro_export]
macro_rules! database_pool {
  (sqlite) => {
    sqlx::Pool<sqlx::Sqlite>
  };
  (mysql) => {
    sqlx::Pool<sqlx::MySql>
  };
  (postgresql) => {
    sqlx::Pool<sqlx::Postgres>
  };
}

/// Returns the appropriate database row type based on the database type.
#[macro_export]
macro_rules! database_row {
    (sqlite) => {
        sqlx::sqlite::SqliteRow
    };
    (mysql) => {
        sqlx::mysql::MySqlRow
    };
    (postgresql) => {
        sqlx::postgres::PgRow
    };
}

/// Returns the appropriate granular operation processing function depending on the database type.
#[macro_export]
macro_rules! granular_operation_fn {
    (sqlite) => {
        $crate::database::sqlite::granular_operation_sqlite
    };
    (mysql) => {
        $crate::database::mysql::granular_operation_mysql
    };
    (postgresql) => {
        $crate::database::postgresql::granular_operation_postgresql
    };
}

/// Returns the appropriate database query fetching function depending on the database type.
#[macro_export]
macro_rules! fetch_query_fn {
    (sqlite) => {
        $crate::database::sqlite::fetch_sqlite_query
    };
    (mysql) => {
        $crate::database::mysql::fetch_mysql_query
    };
    (postgresql) => {
        $crate::database::postgresql::fetch_postgresql_query
    };
}
