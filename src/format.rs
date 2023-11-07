use std::fmt::Display;

/// Formats a table as a Markdown string with space padding for each column.
///
/// # Arguments
///
/// * `headers` - An iterable collection of header items that implement `AsRef<str>` and `Display`.
/// * `rows` - An iterable collection of row iterables. Each row is an iterable collection of items
///             that implement `AsRef<str>` and `Display`.
///
/// # Type Parameters
///
/// * `T` - The type of the header iterable.
/// * `U` - The type of the rows iterable.
/// * `V` - The type of each row, which is itself an iterable collection.
/// * `W` - The type of each item within a row.
///
/// # Returns
///
/// A `String` that represents the formatted table in Markdown syntax with space padding.
pub fn format_table<T, U, V, W>(headers: T, rows: U) -> String
where
    T: IntoIterator,
    T::Item: AsRef<str> + Display,
    U: IntoIterator<Item = V>,
    V: IntoIterator<Item = W>,
    W: AsRef<str> + Display,
{
    let headers: Vec<String> = headers
        .into_iter()
        .map(|h| h.as_ref().to_string())
        .collect();
    let mut column_widths = vec![0; headers.len()];

    // Calculate column widths based on headers
    for (i, header) in headers.iter().enumerate() {
        column_widths[i] = header.len();
    }

    // Calculate column widths based on rows
    let rows: Vec<Vec<String>> = rows
        .into_iter()
        .map(|row| {
            row.into_iter()
                .enumerate()
                .map(|(i, item)| {
                    let item_len = item.as_ref().len();
                    if i < column_widths.len() {
                        column_widths[i] = column_widths[i].max(item_len);
                    }
                    item.as_ref().to_string()
                })
                .collect()
        })
        .collect();

    // Calculate column widths based on rows
    for row in &rows {
        for (i, item) in row.iter().enumerate() {
            column_widths[i] = column_widths[i].max(item.len());
        }
    }

    let mut result = String::new();

    // Format the header with padding
    result.push('|');
    for (i, header) in headers.iter().enumerate() {
        result.push_str(&format!(" {:width$} |", header, width = column_widths[i]));
    }
    result.push('\n');

    // Format the separator
    result.push('|');
    for width in &column_widths {
        result.push_str(&format!(":{:-<width$}:|", "", width = width));
    }
    result.push('\n');

    // Format the rows with padding
    for row in rows {
        result.push('|');
        for (i, item) in row.iter().enumerate() {
            result.push_str(&format!(" {:width$} |", item, width = column_widths[i]));
        }
        result.push('\n');
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the `format_table` function with a simple set of headers and rows for Markdown output with space padding.
    #[test]
    fn test_format_table_markdown() {
        let headers = ["Notebook", "Cell", "ID"];
        let rows = vec![
            vec!["nb1", "cell1", "id1"],
            vec!["nb2", "cell2", "id2"],
            vec!["nb3", "cell3", "id3"],
        ];

        let table_string = format_table(&headers, &rows);
        let expected_output = "\
            | Notebook | Cell  | ID  |\n\
            |:--------:|:-----:|:---:|\n\
            | nb1      | cell1 | id1 |\n\
            | nb2      | cell2 | id2 |\n\
            | nb3      | cell3 | id3 |\n";

        assert_eq!(table_string, expected_output);
    }

    /// Tests the `format_table` function with different types of data (String and &str) for Markdown output with space padding.
    #[test]
    fn test_format_table_mixed_types_markdown() {
        let headers = ["Notebook", "Cell", "ID"];
        let rows = vec![
            vec!["nb1".to_string(), "cell1".to_string(), "id1".to_string()],
            vec!["nb2".to_string(), "cell2".to_string(), "id2".to_string()], // &str literals
            vec!["nb3".to_string(), "cell3".to_string(), "id3".to_string()],
        ];

        let table_string = format_table(&headers, &rows);
        let expected_output = "\
            | Notebook | Cell  | ID  |\n\
            |:--------:|:-----:|:---:|\n\
            | nb1      | cell1 | id1 |\n\
            | nb2      | cell2 | id2 |\n\
            | nb3      | cell3 | id3 |\n";

        assert_eq!(table_string, expected_output);
    }

    /// Tests the `format_table` function with an empty set of rows for Markdown output with space padding.
    #[test]
    fn test_format_table_empty_rows_markdown() {
        let headers = ["Notebook", "Cell", "ID"];
        let rows: Vec<Vec<&str>> = Vec::new();

        let table_string = format_table(&headers, &rows);
        let expected_output = "\
            | Notebook | Cell | ID |\n\
            |:--------:|:----:|:--:|\n";

        assert_eq!(table_string, expected_output);
    }
}
