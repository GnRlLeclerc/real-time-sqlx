//! Implementations for the Tauri backend

use std::collections::HashMap;

use serde::Serialize;
use tauri::ipc::Channel;

use crate::{
    operations::serialize::{object_array_from_value, object_from_value, OperationNotification},
    queries::{serialize::QueryTree, Checkable},
};

/// Process a database operation notification and notify the relevant
/// Tauri channels about the change that occured.
///
/// Returns a list of channel uuid identifiers that errored out and should be pruned.
pub fn process_channel_event<'a, T>(
    channels: &'a HashMap<String, (QueryTree, Channel<serde_json::Value>)>,
    operation: &OperationNotification<T>,
) -> Vec<&'a str>
where
    T: Clone + Serialize,
{
    let serialized_operation = serde_json::to_value(operation).unwrap();
    let data = serialized_operation.get("data").unwrap();

    // Channels that error out, scheduled for pruning at the end.
    let mut failing_channels: Vec<&str> = Vec::new();

    match operation {
        // For single-row operations, we simply push the operation to the channel
        // if the query matches
        OperationNotification::Create { .. } | OperationNotification::Delete { .. } => {
            let object = object_from_value(data.clone()).unwrap();

            for (key, (query, channel)) in channels.iter() {
                if query.check(&object) {
                    // Send an item to the channel, or schedule the channel for deletion
                    if channel.send(serialized_operation.clone()).is_err() {
                        failing_channels.push(key);
                    }
                }
            }
        }
        OperationNotification::Update {
            table,
            data: notif_data,
            id,
        } => {
            // Trick :
            let object = object_from_value(data.clone()).unwrap();

            for (key, (query, channel)) in channels.iter() {
                if query.check(&object) {
                    if channel.send(serialized_operation.clone()).is_err() {
                        failing_channels.push(key);
                    }
                } else {
                    // Because the object has been updated, it is possible that the query
                    // once matched it, but does not anymore. We send a false `Delete`
                    // operation to the frontend to signal that if it ever had this object
                    // in store, it must delete it.
                    let delete_operation = serde_json::to_value(OperationNotification::Delete {
                        table: table.clone(),
                        data: notif_data.clone(),
                        id: id.clone(),
                    })
                    .unwrap();

                    if channel.send(delete_operation).is_err() {
                        failing_channels.push(key);
                    }
                }
            }
        }
        // For multiple-row operations, we check each row individually for matches against
        // the query. We build per-query personalized vectors of matching objects and send
        // them to the corresponding channels
        OperationNotification::CreateMany {
            data: unserialized_data,
            ..
        } => {
            let objects = object_array_from_value(data.clone()).unwrap();

            for (key, (query, channel)) in channels.iter() {
                let mut matching_objects: Vec<T> = Vec::new();
                for (index, object) in objects.iter().enumerate() {
                    if query.check(&object) {
                        matching_objects.push(unserialized_data[index].clone());
                    }
                }

                if !matching_objects.is_empty() {
                    let serialized_operation =
                        serde_json::to_value(OperationNotification::CreateMany {
                            table: "todos".to_string(),
                            data: matching_objects,
                        })
                        .unwrap();
                    if channel.send(serialized_operation).is_err() {
                        failing_channels.push(key);
                    }
                }
            }
        }
    };

    // Return the channels that errored out
    failing_channels
}
