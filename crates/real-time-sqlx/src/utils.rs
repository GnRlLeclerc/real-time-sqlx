use std::{fmt, iter::repeat};

/// Utility function to format a list of displayable items with a specific
/// separator
///
/// Example:
/// 1, 2, 3, condition1 OR condition2 OR condition3
#[inline]
pub(crate) fn format_list<T: fmt::Display>(items: &[T], separator: &str) -> String {
    items
        .iter()
        .map(|item| format!("{}", item).to_string())
        .collect::<Vec<String>>()
        .join(separator)
}

/// Utility function to format an iterator of displayable items with a
/// specific separator
///
/// Example:
/// 1, 2, 3, condition1 OR condition2 OR condition3
#[inline]
pub(crate) fn format_iter<T: fmt::Display, I: IntoIterator<Item = T>>(
    items: I,
    separator: &str,
) -> String {
    items
        .into_iter()
        .map(|item| format!("{}", item).to_string())
        .collect::<Vec<String>>()
        .join(separator)
}

/// Create an owned vector of keys from a JSON object.
/// The vector is not actually "ordered", rather it enables reading the values
/// of multiple similar objects always in the same order for SQL insertion.
#[inline]
pub(crate) fn ordered_keys(object: &serde_json::Map<String, serde_json::Value>) -> Vec<String> {
    object.keys().map(|key| (*key).clone()).collect()
}

/// Convert a string with '?' placeholders to numbered '$1' placeholderss
#[inline]
pub(crate) fn to_numbered_placeholders(query: &str) -> String {
    let mut result = String::new();
    let mut counter = 1;

    for c in query.chars() {
        if c == '?' {
            result.push_str(&format!("${counter}"));
            counter += 1;
        } else {
            result.push(c);
        }
    }

    result
}

/// Create a placeholder string (?, ?, ?) for a given count of placeholders,
/// for one value
#[inline]
pub(crate) fn placeholders(count: usize) -> String {
    let str_placeholders = repeat("?".to_string())
        .take(count)
        .collect::<Vec<String>>()
        .join(", ");

    format!("({str_placeholders})")
}

/// Create a placeholder string (?, ?, ?), (?, ?, ?), (?, ?, ?) for a given
/// count of placeholders, for n values
#[inline]
pub(crate) fn repeat_placeholders(count: usize, n_repeat: usize) -> String {
    repeat(placeholders(count))
        .take(n_repeat)
        .collect::<Vec<String>>()
        .join(", ")
}

/// Sanitize table and column names to avoid SQL injection
/// Only letters, numbers and underscores are allowed. No spaces
#[inline]
pub(crate) fn sanitize_identifier(str: &str) -> String {
    str.replace(|c: char| !c.is_alphanumeric() && c != '_', "")
}

/// Generate an UPDATE statement from a table name and a list of keys
#[inline]
pub(crate) fn update_statement(table: &str, keys: &[String]) -> String {
    let table = sanitize_identifier(table);
    let columns = keys
        .iter()
        .map(|key| format!("\"{}\" = ?", sanitize_identifier(key)))
        .collect::<Vec<String>>()
        .join(", ");

    format!("UPDATE {table} SET {columns} WHERE id = ? RETURNING *")
}

/// Generate an INSERT statement from a table name and a list of keys
#[inline]
pub(crate) fn insert_statement(table: &str, keys: &[String]) -> String {
    let table = sanitize_identifier(table);
    let values_placeholders = placeholders(keys.len());
    let columns = format_iter(keys.iter().map(|s| sanitize_identifier(s)), ", ");

    format!("INSERT INTO {table} ({columns}) VALUES {values_placeholders} RETURNING *")
}

/// Generate an INSERT statement from a table name and a list of keys
/// to insert multiple rows at once
#[inline]
pub(crate) fn insert_many_statement(table: &str, keys: &[String], n_rows: usize) -> String {
    let table = sanitize_identifier(table);
    let values_placeholders = repeat_placeholders(keys.len(), n_rows);
    let columns = format_iter(keys.iter().map(|s| sanitize_identifier(s)), ", ");

    format!("INSERT INTO {table} ({columns}) VALUES {values_placeholders} RETURNING *")
}

/// Generate a DELETE statement from a table name and an id
#[inline]
pub(crate) fn delete_statement(table: &str) -> String {
    let table = sanitize_identifier(table);

    format!("DELETE FROM {table} WHERE id = ? RETURNING *")
}
