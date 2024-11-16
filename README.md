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
     - [Execute SQL operations](#execute-sql-operations)
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

- [Knex](https://knexjs.org): for the typescript query builder frontend.
- [Firestore](https://firebase.google.com/docs/firestore): for the real-time subscription system, and the idea of executing queries directly from the frontend instead of having to implement a backend endpoint for every query.

### What this project is not

- **Not scalable**: query subscriptions are shared on a per-table basis, which is not suitable for multi-consumer applications where you would want to restrict shared updates even more in order to avoid flooding the system with unrelated update signals.
- **Not secure**: there is no security rules to restrict the access to some tables like in firebase. If you wire a table to this system, anyone having access to the frontend code can access anything in it. This is suited for desktop use with local, per-user sqlite databases.
- **Not continuous**: there is currently no way for frontend channels to resubscribe to the backend when their connection breaks, making the project not suitable for situations where you would want to continuously update and restart backend instances with almost seamless subscription transitions in the frontend.

## Installation

For NixOS users, a devshell is provided for this project (which works with Tauri). Run `nix develop`.

As of now, this project is not listed on `crates.io` nor `npmjs.com`.

Install the backend crate in your `Cargo.toml`:

```toml
[dependencies]
real-time-sqlx = { git = "https://github.com/GnRlLeclerc/real-time-sqlx", version = "0.1", features = ["sqlite", "tauri"] }
```

Install the frontend by cloning and building it, then using a local path:

```bash
git clone git@github.com:GnRlLeclerc/real-time-sqlx.git
cd packages/real-time-sqlx
bun install
bun run build
```

In your package.json:

```json
{
  "dependencies": {
    "real-time-sqlx": "file:path/to/real-time-sqlx/packages/real-time-sqlx"
  }
}
```

## Features

This project assumes that the models you intend to wire to it all possess an `id` column that serves as their primary key. It can be of any type.

Features summary:

- **Type-safe query builder** inspired by [Knex](https://knexjs.org/)
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
```

The following functions are exported for you:

- `query`: build SQL `SELECT` queries, fetched using the following functions:
  - `fetch`: fetch a SQL query once
  - `subscribe`: fetch a SQL query and subscribe to its changes
- `execute`: execute SQL operations defined using the following functions:
  - `create`: create a row
  - `createMany`: create many rows at once
  - `update`: update a row
  - `remove`: delete a row

#### Build SQL `SELECT` queries

Select one or multiple rows from a table:

```typescript
import { query } from "real-time-sqlx";

const one = query("model").fetchOne();
const many = query("model").fetchMany();
```

Add and chain conditions:

```typescript
import { query } from "real-time-sqlx";

const q = query("model")
  .where("id", ">", 4)
  .andWhere("title", "ilike", "%hello%");
```

Nest conditions:

```typescript
import { query } from "real-time-sqlx";

const q = query("models")
  .where("id", ">", 4)
  .andWhereCallback((builder) =>
    builder
      .where("title", "ilike", "%hello%")
      .orWhere("title", "ilike", "%hello%"),
  );
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

Fetch some data once:

```typescript
import { query, fetch } from "real-time-sqlx";

const fetchModels = async (): ManyQueryData<Model> => {
  return await fetch<Model>(query("models").fetchMany());
};
```

Fetch some data and subscribe to real-time changes:

```typescript
import { query, subscribe } from "real-time-sqlx";

const unsubscribe = subscribe<Model>(
  query("models").fetchMany(),
  (data, changes) => console.log(JSON.stringify(data)),
);
```

The `unsubscribe` function returned allows you to terminate the subscription early. It is recommended to call it at destruction, although the backend automatically prunes errored / terminated subscription.

#### Execute SQL operations

The return types are explicited here for clarity purposes, but they are actually dynamically inferredfrom the `Model` type and the operation being performed.

Insert a row:

```typescript
import { execute, create } from "real-time-sqlx";

const fetchModels = async (): OperationNotificationCreate<Model> | null => {
  return await execute(
    create<Model>("models", { title: "title", content: "content" }),
  );
};
```

Insert many rows at once:

```typescript
import { execute, createMany } from "real-time-sqlx";

const fetchModels = async (): OperationNotificationCreateMany<Model> | null => {
  return await execute(
    createMany<Model>("models", [
      { title: "title 1", content: "content 1" },
      { title: "title 2", content: "content 2" },
    ]),
  );
};
```

Update a row:

```typescript
import { execute, create } from "real-time-sqlx";

const fetchModels = async (): OperationNotificationUpdate<Model> | null => {
  return await execute(
    update<Model>("models", 3, { title: "new title", content: "new content" }),
  );
};
```

Delete a row:

```typescript
import { execute, remove } from "real-time-sqlx";

const fetchModels = async (): OperationNotificationUpdate<Model> | null => {
  return await execute(remove("models", 42));
};
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
    pub id: i64,
    pub title: String,
    pub content: String,
}
```

Generate the boilerplate code that creates the structures and Tauri commands needed to communicate with the frontend:

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
            execute
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

> [!WARNING]
> Do not call the `real_time_tauri!` macro in your Tauri `lib.rs` file! It will cause issues.

## Roadmap

- [ ] Add support for pagination (`ORDER BY`, `LIMIT`, `OFFSET`)
- [ ] Add model-related type-safety for the frontend builders
- [ ] Expose a raw SQL endpoint for SQL queries not supported by the real-time system, but that you still might want to execute with the same ease.

## Behind the API

TODO: explain with more details how everything works. Link to the backend crate README.
