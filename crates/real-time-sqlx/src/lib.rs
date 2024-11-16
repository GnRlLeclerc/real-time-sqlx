//! Real-time SQLx library

pub mod backends;
pub mod database;
pub mod error;
pub mod macros;
pub mod operations;
pub mod queries;
pub mod utils;

#[cfg(test)]
mod tests;
