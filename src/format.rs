use std::fmt::Display;

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::ASCII_MARKDOWN;
use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::*;

pub fn as_ascii_table<T, U, V, W>(headers: T, rows: U) -> String
where
    T: IntoIterator,
    T::Item: AsRef<str> + Display,
    U: IntoIterator<Item = V>,
    V: IntoIterator<Item = W>,
    W: AsRef<str> + Display,
{
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(headers);
    table.add_rows(rows);
    table.to_string()
}

#[allow(dead_code)]
pub fn as_markdown_table<T, U, V, W>(headers: T, rows: U) -> String
where
    T: IntoIterator,
    T::Item: AsRef<str> + Display,
    U: IntoIterator<Item = V>,
    V: IntoIterator<Item = W>,
    W: AsRef<str> + Display,
{
    let mut table = Table::new();
    table
        .load_preset(ASCII_MARKDOWN)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(headers);
    table.add_rows(rows);
    table.to_string()
}

/// Converts a string to a SQL-friendly identifier following SQLite identifier rules.
///
/// SQLite identifier rules:
/// - Cannot start with a number.
/// - Can only contain alphanumeric characters and underscores.
///
/// # Arguments
///
/// * `input` - The input string to be converted.
///
/// # Example
///
/// ```
/// let input_string = "123 Your Input String!@#";
/// let sql_friendly_identifier = to_sql_friendly_identifier(input_string);
/// println!("{}", sql_friendly_identifier); // Outputs: "your_input_string_"
/// ```
pub fn to_sql_friendly_identifier(input: &str) -> String {
    let re = regex::Regex::new(r"[^a-zA-Z0-9_]+").unwrap();
    let result = re.replace_all(input, "_").to_lowercase();

    // Ensure the identifier does not start with a number.
    if let Some(c) = result.chars().next() {
        if c.is_ascii_digit() {
            return format!("_{}", result);
        }
    }

    result.to_string()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_to_sql_friendly_identifier_alphanumeric() {
        let input_string = "YourInputString123";
        let result = to_sql_friendly_identifier(input_string);
        assert_eq!(result, "yourinputstring123");
    }

    #[test]
    fn test_to_sql_friendly_identifier_starting_with_number() {
        let input_string = "123YourInputString";
        let result = to_sql_friendly_identifier(input_string);
        assert_eq!(result, "_123yourinputstring");
    }

    #[test]
    fn test_to_sql_friendly_identifier_special_characters() {
        let input_string = "Your!@# Input &*() String";
        let result = to_sql_friendly_identifier(input_string);
        assert_eq!(result, "your_input_string");
    }

    #[test]
    fn test_to_sql_friendly_identifier_whitespace() {
        let input_string = "Your Input String";
        let result = to_sql_friendly_identifier(input_string);
        assert_eq!(result, "your_input_string");
    }

    #[test]
    fn test_to_sql_friendly_identifier_empty_string() {
        let input_string = "";
        let result = to_sql_friendly_identifier(input_string);
        assert_eq!(result, "");
    }
}
