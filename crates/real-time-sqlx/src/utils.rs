use std::{fmt, iter::repeat};

/// Utility function to format a list of displayable items with a specific
/// separator
///
/// Example:
/// 1, 2, 3, condition1 OR condition2 OR condition3
#[inline]
pub fn format_list<T: fmt::Display>(items: &[T], separator: &str) -> String {
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
pub fn format_iter<T: fmt::Display, I: IntoIterator<Item = T>>(
    items: I,
    separator: &str,
) -> String {
    items
        .into_iter()
        .map(|item| format!("{}", item).to_string())
        .collect::<Vec<String>>()
        .join(separator)
}

/// Convert a string with '?' placeholders to numbered '$1' placeholderss
#[inline]
pub fn to_numbered_placeholders(query: &str) -> String {
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
pub fn placeholders(count: usize) -> String {
    let str_placeholders = repeat("?".to_string())
        .take(count)
        .collect::<Vec<String>>()
        .join(", ");

    format!("({str_placeholders})")
}

/// Create a placeholder string ($1, $2, 3) for a given count of placeholders,
/// for one value
#[inline]
pub fn numbered_placeholders(count: usize) -> String {
    to_numbered_placeholders(&placeholders(count))
}

/// Create a placeholder string (?, ?, ?), (?, ?, ?), (?, ?, ?) for a given
/// count of placeholders, for n values
#[inline]
pub fn repeat_placeholders(count: usize, n_repeat: usize) -> String {
    repeat(placeholders(count))
        .take(n_repeat)
        .collect::<Vec<String>>()
        .join(", ")
}

/// Create a placeholder string ($1, $2, $3), ($4, $5, $6), ($7, $8, $9) for a
/// given count of placeholders, for n values
#[inline]
pub fn repeat_numbered_placeholders(count: usize, n_repeat: usize) -> String {
    to_numbered_placeholders(&repeat_placeholders(count, n_repeat))
}

/// Sanitize table and column names to avoid SQL injection
/// Only letters, numbers and underscores are allowed. No spaces
#[inline]
pub fn sanitize_identifier(str: &str) -> String {
    str.replace(|c: char| !c.is_alphanumeric() && c != '_', "")
}

/// Generate an UPDATE statement from a table name and a list of keys
#[inline]
pub fn update_statement(table: &str, keys: &[&String]) -> String {
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
pub fn insert_statement(table: &str, keys: &[&String]) -> String {
    let table = sanitize_identifier(table);
    let values_placeholders = placeholders(keys.len());
    let columns = format_iter(keys.iter().map(|s| sanitize_identifier(s)), ", ");

    format!("INSERT INTO {table} ({columns}) VALUES {values_placeholders} RETURNING *")
}

/// Generate an INSERT statement from a table name and a list of keys
/// to insert multiple rows at once
#[inline]
pub fn insert_many_statement(table: &str, keys: &[&String], n_rows: usize) -> String {
    let table = sanitize_identifier(table);
    let values_placeholders = repeat_placeholders(keys.len(), n_rows);
    let columns = format_iter(keys.iter().map(|s| sanitize_identifier(s)), ", ");

    format!("INSERT INTO {table} ({columns}) VALUES {values_placeholders} RETURNING *")
}

/// Generate a DELETE statement from a table name and an id
#[inline]
pub fn delete_statement(table: &str) -> String {
    let table = sanitize_identifier(table);

    format!("DELETE FROM {table} WHERE id = ?")
}
