/** The real-time sqlx entrypoint class */

import { InitialQueryBuilder } from "./builders";
import {
  OperationType,
  type CreateData,
  type FinalValue,
  type Indexable,
  type OperationNotificationCreate,
  type OperationNotificationCreateMany,
  type OperationNotificationDelete,
  type OperationNotificationUpdate,
  type UpdateData,
} from "./types";
import { invoke } from "@tauri-apps/api/core";

export class SQLx<DB extends Record<keyof DB, Indexable>> {
  /** Create a new query on a table */
  select<T extends keyof DB & string>(table: T): InitialQueryBuilder<DB[T]> {
    return new InitialQueryBuilder(table);
  }

  /** Builder to create an entry in a database */
  async create<T extends keyof DB & string>(
    table: T,
    data: CreateData<DB[T]>,
  ): Promise<OperationNotificationCreate<DB[T]> | null> {
    const operation = {
      type: OperationType.Create,
      table,
      data,
    };

    return await invoke("execute", { operation });
  }

  /** Builder to create many entries in a database */
  async createMany<T extends keyof DB & string>(
    table: T,
    data: CreateData<DB[T]>[],
  ): Promise<OperationNotificationCreateMany<DB[T]> | null> {
    const operation = {
      type: OperationType.CreateMany,
      table,
      data,
    };

    return await invoke("execute", { operation });
  }

  /** Builder to update an entry in a database */
  async update<T extends keyof DB & string>(
    table: T,
    id: DB[T]["id"],
    data: UpdateData<DB[T]>,
  ): Promise<OperationNotificationUpdate<DB[T]> | null> {
    const operation = {
      type: OperationType.Update,
      id,
      table,
      data,
    };

    return await invoke("execute", { operation });
  }

  /** Builder to delete an entry in a database */
  async delete<T extends keyof DB & string>(
    table: T,
    id: DB[T]["id"],
  ): Promise<OperationNotificationDelete<DB[T]> | null> {
    const operation = {
      type: OperationType.Delete,
      table,
      id,
    };

    return await invoke("execute", { operation });
  }

  /** Execute a raw prepared SQL query. Returns a list of rows. */
  async rawOne<T = any>(sql: string, values: FinalValue[]): Promise<T | null> {
    return (await invoke<T[]>("raw", { sql, values }))[0] ?? null;
  }

  /** Execute a raw prepared SQL query. Returns a list of rows. */
  async rawMany<T = any>(sql: string, values: FinalValue[]): Promise<T[]> {
    return await invoke("raw", { sql, values });
  }
}
