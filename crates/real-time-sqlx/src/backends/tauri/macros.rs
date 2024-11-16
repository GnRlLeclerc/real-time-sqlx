//! Tauri-related macros

/// Main macro:
/// - Generate the real-time static dispatcher struct that handles channels subscriptions
/// - Generate the tauri commands for the "fetch", "subscribe", "unsubscribe", "execute".
///
/// It should not be used in the lib.rs Tauri entrypoint.
#[macro_export]
macro_rules! real_time_tauri {
    ($db_type:ident, $(($table_name:literal, $struct:ty)),+ $(,)?) => {

        // Generate the real-time dispatcher struct
        $crate::real_time_dispatcher!($db_type, $(($table_name, $struct)),+);

        // Generate the function to statically serialize rows
        $crate::serialize_rows_static!(sqlite, ("todos", Todo), ("again", Todo));

        // Tauri endpoints
        /// Subscribe to a real-time query
        #[tauri::command]
        pub async fn subscribe(
            // Managed by Tauri
            pool: tauri::State<'_, $crate::database_pool!($db_type)>,
            dispatcher: tauri::State<'_, RealTimeDispatcher>,
            // Passed as arguments
            query: $crate::queries::serialize::QueryTree,
            channel_id: String,
            channel: tauri::ipc::Channel<serde_json::Value>,
        ) -> tauri::Result<serde_json::Value> {
            let pool: &$crate::database_pool!($db_type) = &pool;

            // Process the immediate query value to be returned
            let rows = $crate::database::$db_type::fetch_sqlite_query(&query, pool).await;
            let value = serialize_rows_static(&rows, &query.table);

            // Add the channel to the dispatcher
            dispatcher
                .subscribe_channel(&query.table.clone(), &channel_id, query, channel)
                .await;

            Ok(value)
        }

        /// Unsubscribe from a real-time query
        #[tauri::command]
        pub async fn unsubscribe(
            // Managed by Tauri
            dispatcher: tauri::State<'_, RealTimeDispatcher>,
            // Passed as arguments
            channel_id: String,
            table: String,
        ) -> tauri::Result<()> {
            dispatcher.unsubscribe_channel(&table, &channel_id).await;

            Ok(())
        }

        /// Execute a tauri granular operation
        #[tauri::command]
        pub async fn execute(
            // Managed by Tauri
            pool: tauri::State<'_, $crate::database_pool!($db_type)>,
            dispatcher: tauri::State<'_, RealTimeDispatcher>,
            // Passed as arguments
            operation: $crate::operations::serialize::GranularOperation,
        ) -> tauri::Result<()> {
            let pool: &$crate::database_pool!($db_type) = &pool;
            dispatcher.process_operation(operation, pool).await;

            Ok(())
        }

        /// Fetch a query once (without subscription)
        #[tauri::command]
        pub async fn fetch(
            // Managed by Tauri
            pool: tauri::State<'_, $crate::database_pool!($db_type)>,
            // Passed as arguments
            query: $crate::queries::serialize::QueryTree,
        ) -> tauri::Result<serde_json::Value> {
            let pool: &$crate::database_pool!($db_type) = &pool;

            let rows = $crate::database::$db_type::fetch_sqlite_query(&query, pool).await;
            let value = serialize_rows_static(&rows, &query.table);

            Ok(value)
        }
    };
}

/// Generate a real-time static dispatcher struct that can handle subscription channels for
/// different tables. It processes granular operations and updates the channels accordingly.
#[macro_export]
macro_rules! real_time_dispatcher {
    ($db_type:ident, $(($table_name:literal, $struct:ty)),+ $(,)?) => {
        /// Real-time static channel dispatcher for the Tauri backend
        $crate::macros::paste::paste! {
            pub struct RealTimeDispatcher {
                // Define allRwLocked channels for the given tables
                $(
                        pub [<$table_name _channels>]: tokio::sync::RwLock<std::collections::HashMap<String, ($crate::queries::serialize::QueryTree, tauri::ipc::Channel<serde_json::Value>), std::hash::RandomState>>,
                )+
            }
        }

        $crate::macros::paste::paste! {
            impl RealTimeDispatcher {
                /// Implement the generic handler function for all tables and channels
                pub async fn process_operation(
                    &self,
                    operation: $crate::operations::serialize::GranularOperation,
                    pool: &$crate::database_pool!($db_type),
                ) {
                    use $crate::operations::serialize::Tabled;
                    match operation.get_table() {
                        $(
                            $table_name => {
                                // 1. Process the operation and obtain an operation notification
                                let result: Option<$crate::operations::serialize::OperationNotification<$struct>> =
                                    $crate::granular_operation_fn!($db_type)(operation, pool).await;

                                if let Some(result) = result {
                                    // 2. Process the operation notification and update the channels
                                    $crate::backends::tauri::channels::process_event_and_update_channels(
                                        &self.[<$table_name _channels>],
                                        &result,
                                    ).await;
                                }
                            }
                        )+
                        _ => panic!("Table not found"),
                    }
                }

                /// Unsubscribe a channel from the dispatcher
                pub async fn unsubscribe_channel(&self, table: &str, channel_id: &str) {
                    match table {
                        $(
                            $table_name => {
                                let mut channels = self.[<$table_name _channels>].write().await;
                                channels.remove(channel_id);
                            }
                        )+
                        _ => panic!("Table not found"),
                    }
                }

                /// Subscribe a channel to the dispatcher
                pub async fn subscribe_channel(
                    &self,
                    table: &str,
                    channel_id: &str,
                    query: $crate::queries::serialize::QueryTree,
                    channel: tauri::ipc::Channel<serde_json::Value>,
                ) {
                    match table {
                        $(
                            $table_name => {
                                let mut channels = self.[<$table_name _channels>].write().await;
                                channels.insert(channel_id.to_string(), (query, channel));
                            }
                        )+
                        _ => panic!("Table not found"),
                    }
                }

                /// Create a new instance of the dispatcher
                pub fn new() -> Self {
                   RealTimeDispatcher {
                       $(
                           [<$table_name _channels>]: tokio::sync::RwLock::new(std::collections::HashMap::new()),
                       )+
                   }
                }
            }
        }
    };
}
