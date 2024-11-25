# Real-Time SQLx

<div align="center">
  <img src="https://img.shields.io/badge/tauri-v2.0-brightgreen?style=for-the-badge&logo=tauri&logoColor=white" alt="Tauri Version" />
  <img src="https://img.shields.io/badge/rust-orange?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/sqlx-v0.8-red?style=for-the-badge&logo=rust&logoColor=white" alt="SQLx" />
    <img src="https://img.shields.io/badge/typescript-blue?style=for-the-badge&logo=typescript&logoColor=white" alt="TypeScript" />
</div>

<br>

<div align="center">
A simple <a href="https://v2.tauri.app/">Tauri</a> real-time query engine inspired by <a href="https://firebase.google.com/docs/firestore">Firestore</a>
</div>

## Table of Contents

1. [About](#about)
   - [Inspirations](#inspirations)
   - [What this project is not](#what-this-project-is-not)
2. [Installation](#installation)
3. [Features](#features)
   - [Frontend](#frontend)
     - [Build SQL `SELECT` queries](#build-sql-select-queries)
     - [Subscribe to real-time changes](#subscribe-to-real-time-changes)
     - [Paginate SQL queries](#paginate-queries)
     - [Execute SQL operations](#execute-sql-operations)
     - [Execute raw SQL](#execute-raw-sql)
   - [Backend](#backend)
     - [Feature flags](#feature-flags)
     - [Configuration](#configuration)
4. [Roadmap](#roadmap)
5. [Behind the API](#behind-the-api)

## About

This project is a real-time simplified SQL-subset query engine on top of a SQL database that enables you to **subscribe to database changes** in the frontend. It exposes simple query functions in the frontend that make it possible to query your database **directly from the frontend.**
It is primarily thought for simple Tauri dashboard-like applications where you want display data that is always in sync with the local database, without having to handle cache invalidation, refetches, etc.

This project relies on the [sqlx](https://github.com/launchbadge/sqlx) rust crate for database interaction, which you can use for more complex SQL operations that are not supported by this project.

### Inspirations

- [Kysely](https://kysely.dev/): for the typescript query builder frontend.
- [Firestore](https://firebase.google.com/docs/firestore): for the real-time subscription system, and the idea of executing queries directly from the frontend instead of having to implement a backend endpoint for every query.

### What this project is not

- **Not scalable**: query subscriptions are shared on a per-table basis, which is not suitable for multi-consumer applications where you would want to restrict shared updates even more in order to avoid flooding the system with unrelated update signals.
- **Not secure**: there is no security rules to restrict the access to some tables like in firebase. If you wire a table to this system, anyone having access to the frontend code can access anything in it. This is suited for desktop use with local, per-user sqlite databases.
- **Not continuous**: there is currently no way for frontend channels to resubscribe to the backend when their connection breaks, making the project not suitable for situations where you would want to continuously update and restart backend instances with almost seamless subscription transitions in the frontend.

## Installation

For NixOS users, a devshell is provided for this project (which works with Tauri). Run `nix develop`.

Add the backend crate in your `Cargo.toml`:

```toml
[dependencies]
real-time-sqlx = { version = "0.1", features = ["sqlite", "tauri"] }
```

Install the frontend with your favorite package manager:

```bash
bun add real-time-sqlx
```

## Features

This project assumes that the models you intend to wire to it all possess an `id` column that serves as their primary key. It can be of any type.

Features summary:

- **Type-safe query builder** inspired by [Kysely](https://kysely.dev/)
- **Real-time database subscriptions** inspired by [Firestore](https://firebase.google.com/docs/firestore)
- Execute SQL queries directly from the frontend with _very little boilerplate_.

### Frontend

Define your typescript models:

```typescript
import { Indexable } from "real-time-sqlx";

interface Model extends Indexable {
  id: number;
  title: string;
  content: string;
}

interface Todo extends Indexable {
  id: string; // Strings are valid IDs
  title: string;
  content: string;
}
```

Then define a database interface that contains all of your models:

```typescript
interface Database {
  models: Model; // Name the attributes with the same name as the corresponding table!
  todos: Todos;
}
```

Finally, you can instanciate a `SQLx instance`. It will ensure that your queries are valid in a type-safe way:

```typescript
import { SQLx } from "real-time-sqlx";

// Export it through your app
export const sqlx = new SQLx<Database>("sqlite");
```

The following methods are made available by this instance:

- `select`: build SQL `SELECT` queries, fetched using the following functions:
  - `fetchOne`/`fetchMany`: fetch a SQL query once
  - `subscribeOne`/`subscribeMany`: fetch a SQL query and subscribe to its changes
- `create`: create a row
- `createMany`: create many rows at once
- `update`: update a row
- `remove`: delete a row
- `rawOne`: execute a sql query, returning one row at most
- `rawMany`: execute a sql query, returning all rows

#### Build SQL `SELECT` queries

Select one or multiple rows from a table:

```typescript
// The return types explicited here are inferred automatically!
const one: SingleQueryData<Model> = await sqlx.select("model").fetchOne();
const many: ManyQueryData<Model> = await sqlx.select("model").fetchMany();
```

Add and chain conditions:

```typescript
const { data } = await sqlx
  .select("model")
  .where("id", ">", 4)
  .and("title", "ilike", "%hello%")
  .fetchOne();
```

Nest conditions:

```typescript
const { data } = await sqlx
  .select("models")
  .where("id", ">", 4)
  .andCallback((builder) =>
    builder.where("title", "ilike", "%hello%").or("title", "ilike", "%hello%"),
  )
  .fetchMany();
```

Supported SQL operators:

- `=`
- `!=`
- `<`
- `>`
- `<=`
- `>=`
- `like`
- `ilike`
- `in`

Unsupported SQL conditions:

- `JOIN` (too complex to implement, not suitable for real-time subscriptions)
- `ORDER BY`, `LIMIT`, `OFFSET` (planned for a future release)

#### Subscribe to real-time changes

Fetch some data and subscribe to real-time changes:

```typescript
const unsubscribe = sqlx
  .select("models")
  .subscribeMany(
    (data: Model[], updates: OperationNotification<Model> | null) =>
      console.log(JSON.stringify(data)),
  );
```

The `unsubscribe` function returned allows you to terminate the subscription early. It is recommended to call it at destruction, although the backend automatically prunes errored / terminated subscriptions.

#### Paginate queries

Pagination options are supported for SQL queries. By default, if pagination options are specified without the `orderBy` clause, the results will be ordered by `id DESC` (most recent entries first, for autoincrement primary keys).

```typescript
const { data } = await sqlx.select("model").fetchOne({
  perPage: 10,
  offset: 0,
  orderBy: { column: "id", order: "desc" },
});
```

```typescript
const { data } = await sqlx.select("model").fetchMany({
  perPage: 10,
  offset: 0,
  orderBy: { column: "id", order: "desc" },
});
```

You can subscribe to real-time paginated queries! The `paginate` method returns an unsubscribe function as well as a `fetchMore` function to iterate over the data.

```typescript
const [unsubscribe, fetchMore] = sqlx.select("model").paginate(
  {
    perPage: 10,
    offset: 0,
    orderBy: { column: "id", order: "desc" },
  },
  (data: Model[], updates: OperationNotification<Model> | null) =>
    console.log(JSON.stringify(data)),
);

const affectedRowCount = await fetchMore();
```

#### Execute SQL operations

The return types are explicited here for clarity purposes, but they are actually dynamically inferred from the `Model` type and the operation being performed.

Insert a row:

```typescript
const notification: OperationNotificationCreate<Model> | null =
  await sqlx.create("models", { title: "title", content: "content" });
```

Insert many rows at once:

```typescript
const notification: OperationNotificationCreateMany<Model> | null =
  await sqlx.createMany("models", [
    { title: "title 1", content: "content 1" },
    { title: "title 2", content: "content 2" },
  ]);
```

Update a row:

```typescript
const notification: OperationNotificationUpdate<Model> | null =
  await sqlx.update(
    "models",
    3, // ID
    { title: "new title", content: "new content" },
  );
```

Delete a row:

```typescript
const notification: OperationNotificationUpdate<Model> | null =
  await sqlx.delete(
    "models",
    42, // ID
  );
```

#### Execute raw SQL

Because these simple query builders do not include all possible SQL operations, 2 more methods exist to execute raw prepared SQL queries.

Execute a SQL query and return at most one row (or `null` if nothing is returned):

```typescript
const data: Model | null = await sqlx.rawOne(
  "SELECT * from models where id = ?",
  [42],
);
```

Execute a SQL query and return all found rows:

```typescript
const data: Model[] = await sqlx.rawMany("SELECT * from models where id > ?", [
  1,
]);
```

### Backend

#### Feature Flags

The rust backend exposes the following feature flags. It is intended for use with Tauri.

- `postgres`: PostgreSQL database compatibility
- `mysql`: MySQL database compatibility
- `sqlite`: Sqlite database compatibility
- `tauri`: Complete Tauri integration

#### Configuration

Create your rust database models (note that the models do not need to implement `Deserialize`):

```rust
#[derive(sqlx::FromRow, serde::Serialize, Clone)]
pub struct Model {
    pub id: i64,
    pub title: String,
    pub content: String,
}


#[derive(sqlx::FromRow, serde::Serialize, Clone)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub content: String,
}
```

Generate the boilerplate code that creates the structures and Tauri commands needed to communicate with the frontend.
You have to specify `(sql table name, rust struct)` pairs that will be matched together by the engine.

```rust
real_time_sqlx::real_time_tauri!(sqlite, ("models", Model), ("todos", Todo));  // For Sqlite (recommended for Tauri)
```

Although `Sqlite`, `MySQL` and `PostgreSQL` are all supported (courtesy of `sqlx`), only `Sqlite` is recommended for use.

This macro generates a `RealTimeDispatcher` struct that will handle the `Channel` connections to the frontend, perform SQL operations,
and notify the relevant channels.

It also creates the following Tauri commands:

- `fetch`
- `execute`
- `subscribe`
- `unsubscribe`
- `raw`

These Tauri commands expect 2 states to be managed by Tauri:

- A database pool (`sqlx::Pool<sqlx::Sqlite>` for instance)
- A real-time dispatcher

Your `lib.rs` should look like this:

```rust

async fn create_database_pool() -> Result<Pool<Sqlite>, sqlx::Error> {
  let options = SqliteConnectOptions::new();

  // Need to do this because else `log_statements` is not detected
  let options = options
    .filename("/path/to/your/database.db")
    .disable_statement_logging()
    .create_if_missing(true);

  SqlitePoolOptions::new()
    .max_connections(5)
    .connect_with(options)
    .await
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Create database pool
    let pool =
        async_runtime::block_on(create_database_pool()).expect("Failed to create database pool");

    // Run your migrations with sqlx::migrate!

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(pool)
        .manage(RealTimeDispatcher::new())
        .invoke_handler(tauri::generate_handler![
            // Include the generated Tauri commands
            fetch,
            subscribe,
            unsubscribe,
            execute,
            raw
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

> [!WARNING]
> Do not call the `real_time_tauri!` macro in your Tauri `lib.rs` file! It will cause issues.

## Roadmap

- [x] Add support for pagination (`ORDER BY`, `LIMIT`, `OFFSET`)
- [x] Add model-related type-safety for the frontend builders
- [x] Expose a raw SQL endpoint for SQL queries not supported by the real-time system, but that you still might want to execute with the same ease.
- [ ] Add end-to-end testing.
- [ ] Add support for other `id` names (using an optional additional argument)

## Behind the API

See the [Backend README](./crates/real-time-sqlx/README.md).
