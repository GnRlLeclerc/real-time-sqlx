/** Helper functions for subscriptions */

import { Channel, invoke } from "@tauri-apps/api/core";
import { v4 as uuidv4 } from "uuid";
import { ConditionNone, type Condition } from "./conditions";
import {
  OperationType,
  QueryReturnType,
  type Indexable,
  type ManyQueryData,
  type OperationNotification,
  type SerializedQuery,
  type SingleQueryData,
} from "./types";

// ************************************************************************* //
//                                  TYPES                                    //
// ************************************************************************* //

/** Unsubscribe to a channel */
export type UnsubscribeFn = () => void;

export type UpdateSingleFn<T extends Indexable> = (
  data: T | null,
  updates: OperationNotification<T> | null,
) => void;

export type UpdateManyFn<T extends Indexable> = (
  data: T[],
  updates: OperationNotification<T> | null,
) => void;

// ************************************************************************* //
//                              IMPLEMENTATIONS                              //
// ************************************************************************* //

/** Implementation of the subscription to a single optional value */
export const subscribeOne = <T extends Indexable>(
  table: string,
  condition: Condition,
  callback: UpdateSingleFn<T>,
): UnsubscribeFn => {
  // Generate a unique subscription ID and an unsubscription function.
  const channelId = uuidv4();
  const unsubscribe = () => invoke("unsubscribe", { channelId, table });

  // Create the channel to receive updates
  const channel = new Channel<OperationNotification<T>>();

  // Create the internal data store
  let internalData: T | null = null;

  // Set the channel callback
  channel.onmessage = (update) => {
    // Update cached internal data
    switch (update.type) {
      case OperationType.Delete:
        internalData = null;
        break;
      case OperationType.Create:
      case OperationType.Update:
        if (internalData !== null && internalData.id !== update.data.id) {
          break;
        }
        internalData = update.data;
        break;
      case OperationType.CreateMany:
        for (const data of update.data) {
          if (internalData !== null && internalData.id !== data.id) {
            break;
          }
          internalData = data;
        }
        break;
    }

    // Call the callback with the updated data
    callback(internalData, update);
  };

  // Send the initial query to the database
  const query: SerializedQuery = {
    return: QueryReturnType.Single,
    table,
    condition: condition instanceof ConditionNone ? null : condition.toJSON(),
  };
  invoke<SingleQueryData<T>>("subscribe", {
    query,
    channel,
    channelId,
  }).then(({ data }) => {
    internalData = data;
    // Call the callback with the initial data
    callback(internalData, null);
  });

  // Return the unsubscribe function
  return unsubscribe;
};

/** Implementation of the subscription to a list of values */
export const subscribeMany = <T extends Indexable>(
  table: string,
  condition: Condition,
  callback: UpdateManyFn<T>,
): UnsubscribeFn => {
  // Generate a unique subscription ID and an unsubscription function.
  const channelId = uuidv4();
  const unsubscribe = () => invoke("unsubscribe", { channelId, table });

  // Create the channel to receive updates
  const channel = new Channel<OperationNotification<T>>();

  // Create the internal data
  let internalData: T[] = [];
  let internalMap: Record<string | number, T> = {};

  // Set the callback
  channel.onmessage = (update) => {
    // Update cached internal data
    switch (update.type) {
      case OperationType.Delete:
        delete internalMap[update.data.id as string | number];
        internalData = Object.values(internalMap);
        break;
      case OperationType.Create:
      case OperationType.Update:
        internalMap[update.data.id as string | number] = update.data;
        internalData = Object.values(internalMap);
        break;
      case OperationType.CreateMany:
        for (const data of update.data) {
          internalMap[data.id as string | number] = data;
        }
        internalData = Object.values(internalMap);
        break;
    }

    // Call the callback
    callback(internalData, update);
  };

  // Send the query to the database
  const query: SerializedQuery = {
    return: QueryReturnType.Many,
    table,
    condition: condition instanceof ConditionNone ? null : condition.toJSON(),
  };
  invoke<ManyQueryData<T>>("subscribe", {
    query,
    channel,
    channelId,
  }).then(({ data }) => {
    // Set the initial internal data
    data.forEach((d) => (internalMap[d.id as string | number] = d));
    internalData = Object.values(internalMap);

    // Call the callback with the initial data
    callback(internalData, null);
  });

  // Return the unsubscribe function
  return unsubscribe;
};
