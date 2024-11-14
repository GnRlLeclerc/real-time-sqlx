# Real-Time SQLx

Rust backend for the real-time query engine.
You can run the tests implemented against the SQLite backend with `cargo test`.

Example usage:

Define structs that can be serialized to JSON and built from sqlx rows:

```rust
/// A table model
#[derive(sqlx::FromRow, serde::Serialize)]
pub struct Model {
  pub id: i32,
  pub content: String,
}
```

Execute serialized queries and serialize the results back to JSON.

```rust
// Load a serialized query
let query: QueryTree = serde_json::from_str(
    r#"{
        "return": "single",
        "table": "users",
        "condition": {
            "type": "single",
            "constraint": {
                "column": "id",
                "operator": "=",
                "value": {
                    "final": 1
                }
            }
        }
    }"#,
)
.unwrap();

// Fetch rows (depending on your preferred database)
let rows = fetch_sqlite_query(&query, &database_pool);
let rows = fetch_mysql_query(&query, &database_pool);
let rows = fetch_postgres_query(&query, &database_pool);

// Serialize the data to JSON again, ready to be sent to the frontend
let json_value = serialize_rows::<Model, SqliteRow>(rows);
let json_value = serialize_rows::<Model, MysqlRow>(rows);
let json_value = serialize_rows::<Model, PgRow>(rows);
```

You can also statically serialize rows using different models using the table name as a discriminant.

```rust
match query.table {
    "model1" => serialize_rows::<Model1, PgRow>(rows),
    "model2" => serialize_rows::<Model2, PgRow>(rows),
    "model3" => serialize_rows::<Model3, PgRow>(rows),
    _ => panic!("Unknown table name"), // ...or your preferred error handling
}
```

Execute serialized create, create many, update and delete operations:

```rust
let operation = GranularOperation::Create<Model> {
    table = "model1".to_string(),
    data = Model { content: "content".to_string() },
};

// The result contains returned data about the created row,
// to be sent to the frontend for processing.
let result = granular_operation_sqlite(operation, &pool).await;

```
