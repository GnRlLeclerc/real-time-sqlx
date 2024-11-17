/** Frontend type helpers, and implementation for Tauri channels. */

import { FinalQuerySingle, type FinalQueryMany } from "./builders";
import type {
  GranularOperationCreate,
  GranularOperationCreateMany,
  GranularOperationDelete,
  GranularOperationUpdate,
  Indexable,
  ManyQueryData,
  OperationNotification,
  OperationNotificationCreate,
  OperationNotificationCreateMany,
  OperationNotificationDelete,
  OperationNotificationUpdate,
  SingleQueryData,
} from "./types";

export * from "./frontend/tauri";

// ************************************************************************* //
//                            FUNCTION OVERLOADS                             //
// ************************************************************************* //

// Function overloads designed to mimick the serialized return types of the
// rust backend depending on the inputs.

/** Overloaded fetch function definition.
 *
 */
export type FetchFn = {
  <T>(query: FinalQuerySingle<T>): Promise<SingleQueryData<T>>;
  <T>(query: FinalQueryMany<T>): Promise<ManyQueryData<T>>;
};

/** Overloaded operation function definition. If the returned operation is null,
 * it failed.
 */
export type OperationFn = {
  <T extends Indexable>(
    operation: GranularOperationCreate<T>,
  ): Promise<OperationNotificationCreate<T> | null>;
  <T extends Indexable>(
    operation: GranularOperationCreateMany<T>,
  ): Promise<OperationNotificationCreateMany<T> | null>;
  <T extends Indexable>(
    operation: GranularOperationUpdate<T>,
  ): Promise<OperationNotificationUpdate<T> | null>;
  <T extends Indexable>(
    operation: GranularOperationDelete,
  ): Promise<OperationNotificationDelete<T> | null>;
};

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

/** Overloaded subscribe function definition */
export type SubscribeFn = {
  <T extends Indexable>(
    query: FinalQuerySingle<T>,
    callback: UpdateSingleFn<T>,
  ): UnsubscribeFn;
  <T extends Indexable>(
    query: FinalQueryMany<T>,
    callback: UpdateManyFn<T>,
  ): UnsubscribeFn;
};
