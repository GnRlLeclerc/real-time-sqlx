/** Predefined methods for the Tauri frontend.
 * The `@tauri-apps/api` package is required
 */

import { FinalQuerySingle, type FinalQueryMany } from "../builders";
import type {
  FetchFn,
  OperationFn,
  SubscribeFn,
  UnsubscribeFn,
  UpdateManyFn,
  UpdateSingleFn,
} from "../frontend";
import {
  OperationType,
  type GranularOperation,
  type Indexable,
  type ManyQueryData,
  type OperationNotification,
  type SingleQueryData,
} from "../types";
import { v4 as uuidv4 } from "uuid";

/** Fetch the invoke method asynchronously */
const getInvoke = async (): Promise<
  typeof import("@tauri-apps/api/core").invoke
> => (await import("@tauri-apps/api/core")).invoke;

/** Fetch the tauri Channel class asynchronously */
const getChannel = async (): Promise<
  typeof import("@tauri-apps/api/core").Channel
> => (await import("@tauri-apps/api/core")).Channel;

// ************************************************************************* //
//                         FETCH & UPDATE FUNCTIONS                          //
// ************************************************************************* //

const tauriExecute = async (operation: GranularOperation) => {
  const invoke = await getInvoke();
  return await invoke("execute", { operation });
};

/** Execute a granular operation in database.
 * Assumes that the corresponding Tauri endpoint is named `execute`.
 */
export const execute = tauriExecute as OperationFn;

const tauriFetch = async (query: FinalQuerySingle | FinalQueryMany) => {
  const invoke = await getInvoke();
  return await invoke("fetch", { query: query.toJSON() });
};

/** Fetch data once from the database.
 * Assumes that the corresponding Tauri endpoint is named `fetch`.
 */
export const fetch = tauriFetch as FetchFn;

// ************************************************************************* //
//                           SUBSCRIPTION FUNCTION                           //
// ************************************************************************* //

const tauriSubscribe = <T extends Indexable>(
  query: FinalQuerySingle | FinalQueryMany,
  callback: UpdateSingleFn<T> | UpdateManyFn<T>,
): UnsubscribeFn => {
  if (query instanceof FinalQuerySingle) {
    return subscribeSingle(query, callback as UpdateSingleFn<T>);
  } else {
    return subscribeMany(query, callback as UpdateManyFn<T>);
  }
};

/** Subscribes to a query in the database.
 * Assumes that the Tauri subscription endpoint is named `subscribe`,
 * and that the Tauri unsubscription endpoint is named `unsubscribe`.
 */
export const subscribe = tauriSubscribe as SubscribeFn;

/** Implementation of the subscription to a single optional value */
const subscribeSingle = <T extends Indexable>(
  finalQuery: FinalQuerySingle,
  callback: UpdateSingleFn<T>,
): UnsubscribeFn => {
  // Generate a unique subscription ID and an unsubscription function.
  const id = uuidv4();
  const unsubscribe = () => {
    getInvoke().then((invoke) =>
      invoke("unsubscribe", { id, table: finalQuery.table }),
    );
  };

  // Do the rest of the operations in an async function
  // that will be called synchronously at the end.
  const build = async () => {
    const Channel = await getChannel();
    const invoke = await getInvoke();

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

    // Send the query to the database
    const query = finalQuery.toJSON();
    const { data } = await invoke<SingleQueryData<T>>("subscribe", {
      query,
      channel,
      id,
    });
    internalData = data;

    // Call the callback with the initial data
    callback(internalData, null);
  };

  // Start the async subscription build process
  build();

  // Return the unsubscribe function
  return unsubscribe;
};

/** Implementation of the subscription to a list of values */
const subscribeMany = <T extends Indexable>(
  finalQuery: FinalQueryMany,
  callback: UpdateManyFn<T>,
): UnsubscribeFn => {
  // Generate a unique subscription ID and an unsubscription function.
  const id = uuidv4();
  const unsubscribe = () => {
    getInvoke().then((invoke) =>
      invoke("unsubscribe", { id, table: finalQuery.table }),
    );
  };

  // Do the rest of the operations in an async function
  // that will be called synchronously at the end.
  const build = async () => {
    const Channel = await getChannel();
    const invoke = await getInvoke();

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
    const query = finalQuery.toJSON();
    const { data } = await invoke<ManyQueryData<T>>("subscribe", {
      query,
      channel,
      id,
    });

    // Set the initial internal data
    data.forEach((d) => (internalMap[d.id as string | number] = d));
    internalData = Object.values(internalMap);

    // Call the callback with the initial data
    callback(internalData, null);
  };

  // Start the async subscription build process
  build();

  // Return the unsubscribe function
  return unsubscribe;
};
