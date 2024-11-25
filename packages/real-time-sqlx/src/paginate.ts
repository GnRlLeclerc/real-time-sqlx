/** Subscription to a paginated query */

import { Channel, invoke } from "@tauri-apps/api/core";
import { ConditionNone, type Condition } from "./conditions";
import type { FetchMoreFn, UnsubscribeFn, UpdateManyFn } from "./subscribe";
import {
  OperationType,
  QueryReturnType,
  type Indexable,
  type ManyQueryData,
  type OperationNotification,
  type OrderBy,
  type PaginateOptions,
  type SerializedQuery,
} from "./types";
import { v4 as uuidv4 } from "uuid";

const DEFAULT_ORDER = { column: "id", order: "desc" } as const;

/** Sort an array of objects by a key.
 * If no option is given, sorts by decreasing index
 */
const sortBy = <T extends Indexable>(
  array: T[],
  orderBy: OrderBy<T> | null = null,
): T[] => {
  const { column, order } = orderBy ?? DEFAULT_ORDER;
  return array.sort((a, b) => {
    if (order === "asc") {
      return a[column] >= b[column] ? 1 : -1;
    } else {
      return a[column] <= b[column] ? 1 : -1;
    }
  });
};

/** Check if an incoming item is in the pagination range yet by comparing its discriminant
 * field to that of the last item in the current pagination range
 */
const isInRange = <T extends Indexable>(
  item: T,
  lastValue: any | null,
  orderBy: OrderBy<T> | null = null,
) => {
  const { column, order } = orderBy ?? DEFAULT_ORDER;

  if (lastValue === null) {
    return true;
  }
  if (order === "asc") {
    return item[column] <= lastValue;
  } else {
    return item[column] >= lastValue;
  }
};

/** Update the last accepted discriminant value for the current pagination range */
const updateDiscriminant = <T extends Indexable>(
  sortedValues: T[],
  orderBy: OrderBy<T> | null = null,
) => {
  if (sortedValues.length === 0) {
    return null;
  }
  const column = orderBy?.column ?? DEFAULT_ORDER.column;

  return sortedValues[sortedValues.length - 1][column];
};

/** Implementation of the subscription to a list of values */
export const paginate = <T extends Indexable>(
  table: string,
  condition: Condition,
  options: PaginateOptions<T>,
  callback: UpdateManyFn<T>,
): [UnsubscribeFn, FetchMoreFn] => {
  // Generate a unique subscription ID and an unsubscription function.
  const channelId = uuidv4();
  const unsubscribe = () => invoke("unsubscribe", { channelId, table });

  // Create the channel to receive updates
  const channel = new Channel<OperationNotification<T>>();

  // Create the internal data
  let internalData: T[] = [];
  let internalMap: Record<string | number, T> = {};
  let lastDiscriminant: any = null;
  let anyLeft = true;

  // Set the callback
  channel.onmessage = (update) => {
    // Update cached internal data
    switch (update.type) {
      case OperationType.Delete:
        if (internalMap[update.data.id as string | number] === undefined) {
          return;
        }

        delete internalMap[update.data.id as string | number];
        internalData = sortBy(Object.values(internalMap), options.orderBy);
        lastDiscriminant = updateDiscriminant(internalData, options.orderBy);
        break;

      case OperationType.Create:
      case OperationType.Update:
        if (!isInRange(update.data, lastDiscriminant, options.orderBy)) {
          anyLeft = true;
          return;
        }

        internalMap[update.data.id as string | number] = update.data;
        internalData = sortBy(Object.values(internalMap), options.orderBy);
        lastDiscriminant = updateDiscriminant(internalData, options.orderBy);
        break;

      case OperationType.CreateMany:
        let valid = 0;
        for (const data of update.data) {
          if (!isInRange(data, lastDiscriminant, options.orderBy)) {
            anyLeft = true;
            continue;
          }

          internalMap[data.id as string | number] = data;
          valid++;
        }
        if (valid === 0) {
          return;
        }

        internalData = sortBy(Object.values(internalMap), options.orderBy);
        lastDiscriminant = updateDiscriminant(internalData, options.orderBy);
        break;
    }

    // Call the callback for the paths that have not exited yet (i.e. a relevant operation happened)
    callback(internalData, update);
  };

  // Send the query to the database with the initial pagination parameters
  const query: SerializedQuery<T> = {
    return: QueryReturnType.Many,
    table,
    condition: condition instanceof ConditionNone ? null : condition.toJSON(),
    paginate: options ?? null,
  };
  invoke<ManyQueryData<T>>("subscribe", {
    query,
    channel,
    channelId,
  }).then(({ data }) => {
    // Set the initial internal data
    data.forEach((d) => (internalMap[d.id as string | number] = d));
    internalData = sortBy(Object.values(internalMap), options.orderBy);
    lastDiscriminant = updateDiscriminant(internalData, options.orderBy);

    // Call the callback with the initial data
    callback(internalData, null);
  });

  const fetchMore: FetchMoreFn = async () => {
    // Avoid backend calls if we already know that there is nothing left.
    if (!anyLeft) {
      return 0;
    }

    // Update the pagination options
    const paginate: PaginateOptions<T> = {
      orderBy: options.orderBy,
      perPage: options.perPage,
      offset: internalData.length + (options.offset ?? 0),
    };

    const query: SerializedQuery<T> = {
      return: QueryReturnType.Many,
      table,
      condition: condition instanceof ConditionNone ? null : condition.toJSON(),
      paginate,
    };

    const { data } = await invoke<ManyQueryData<T>>("fetch", {
      query,
      channel,
      channelId,
    });

    if (data.length === 0) {
      anyLeft = false;
    }

    // Merge the new data with the existing one
    data.forEach((d) => (internalMap[d.id as string | number] = d));
    internalData = sortBy(Object.values(internalMap), options.orderBy);
    lastDiscriminant = updateDiscriminant(internalData, options.orderBy);

    // Call the callback with the new data
    callback(internalData, null);

    // Return the amount of affected rows
    return data.length;
  };

  // Return the unsubscribe function
  return [unsubscribe, fetchMore];
};
