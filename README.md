# Real-Time SQLx

A simple real-time query engine inspired by [Firestore](https://firebase.google.com/docs/firestore) subscriptions, with a Typescript frontend and a [sqlx](https://github.com/launchbadge/sqlx)-based Rust backend.

Note that this project only implements a subset of SQL for simplicity reasons (both because implementing features such as `JOIN` would add a lot of complexity, but also because you don't really need real-time for a complex `JOIN` query).

## Repository Structure

```
├── crates    # Rust crates
│   └── real-time-sqlx # Real-time queries backend
│
└── packages  # Typescript packages
    └── real-time-sqlx  # Real-time queries frontend
```
