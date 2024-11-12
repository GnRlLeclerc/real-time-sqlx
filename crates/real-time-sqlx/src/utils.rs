use std::fmt;

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
