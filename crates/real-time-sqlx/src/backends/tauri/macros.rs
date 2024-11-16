//! Tauri-related macros

/// Generate a real-time static dispatcher struct that can handle subscription channels for
/// different tables. It processes granular operations and updates the channels accordingly.
pub mod macros {
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
}
