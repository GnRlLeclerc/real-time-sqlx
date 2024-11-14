# Real-Time SQLx

Frontend implementation of the real-time sqlx engine, in `Typescript`.
The package manager is `bun`.

Install dependencies:

```bash
bun install
```

Develop:

```bash
bun run build --watch
```

## Exemple usage

The goal of this package is to build queries that can be serialized to JSON for sending to a rust backend that can interpret and execute them.

For simplicity purposes, this package and the backend crate assume that the primary key on your objects is **always** named `id`. You are not forced to serve your whole database through this API, but the tables you intend to must conform to this rule.

Select queries:

```ts
const json_query = query("my_table")
  .where("id", "=", "1")
  .andWhereCallback((builder) =>
    builder.where("my_column", ">", 42).orWhere("my_column", "<", 120),
  )
  .toJSON();
```

Create query:

```ts
const json_query = create<MyModel>("my_table", { data }).toJSON();
```

Update query:

```ts
const json_query = update<MyModel>("my_table", 42, { updated_data }).toJSON();
```

Delete query:

```ts
const json_query = remove("my_table", 42);
```
