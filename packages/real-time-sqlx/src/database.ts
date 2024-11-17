/** The real-time sqlx entrypoint class */

import {
  DummyDriver,
  Kysely,
  MysqlAdapter,
  MysqlIntrospector,
  MysqlQueryCompiler,
  PostgresAdapter,
  PostgresIntrospector,
  PostgresQueryCompiler,
  SqliteAdapter,
  SqliteIntrospector,
  SqliteQueryCompiler,
  type DatabaseIntrospector,
  type DialectAdapter,
} from "kysely";
import { InitialQueryBuilder } from "./builders";
import {
  OperationType,
  type CreateData,
  type Indexable,
  type OperationNotificationCreate,
  type OperationNotificationCreateMany,
  type OperationNotificationDelete,
  type OperationNotificationUpdate,
  type UpdateData,
} from "./types";
import type { QueryCompiler } from "kysely";
import { invoke } from "@tauri-apps/api/core";

type SupportedDB = "sqlite" | "mysql" | "postgres";

const adapter = (type: SupportedDB): DialectAdapter => {
  switch (type) {
    case "sqlite":
      return new SqliteAdapter();
    case "mysql":
      return new MysqlAdapter();
    case "postgres":
      return new PostgresAdapter();
    default:
      throw new Error("Unsupported database");
  }
};

const introspector = (
  type: SupportedDB,
  db: Kysely<any>,
): DatabaseIntrospector => {
  switch (type) {
    case "sqlite":
      return new SqliteIntrospector(db);
    case "mysql":
      return new MysqlIntrospector(db);
    case "postgres":
      return new PostgresIntrospector(db);
    default:
      throw new Error("Unsupported database");
  }
};

const compiler = (type: SupportedDB): QueryCompiler => {
  switch (type) {
    case "sqlite":
      return new SqliteQueryCompiler();
    case "mysql":
      return new MysqlQueryCompiler();
    case "postgres":
      return new PostgresQueryCompiler();
    default:
      throw new Error("Unsupported database");
  }
};

export class SQLx<DB extends Record<keyof DB, Indexable>> {
  private db: Kysely<DB>;

  constructor(type: SupportedDB) {
    this.db = new Kysely<DB>({
      dialect: {
        createAdapter: () => adapter(type),
        createDriver: () => new DummyDriver(),
        createIntrospector: (db) => introspector(type, db),
        createQueryCompiler: () => compiler(type),
      },
    });
  }

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

  // TODO: raw sql with kysely.
}
