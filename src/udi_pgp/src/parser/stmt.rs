use derive_new::new;
use sqlparser::ast::Statement;

/// Contains metadata about a specific column in a SQL query, including its name, type, and optional alias.
#[derive(Debug, Clone, new, PartialEq)]
pub struct ColumnMetadata {
    /// Name of the column.
    pub name: String,
    /// Type of expression the column represents (e.g., Standard, Function, Binary).
    pub expr_type: ExpressionType,
    /// Optional alias assigned to the column in the query.
    pub alias: Option<String>,
}

impl Default for ColumnMetadata {
    fn default() -> Self {
        ColumnMetadata {
            name: String::new(),
            expr_type: ExpressionType::Standard,
            alias: None,
        }
    }
}

/// Enum representing the types of expressions a column in a SQL query can have.
/// Corresponding directly to the types in the statement
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionType {
    /// A standard column or field.
    Standard,
    /// A column resulting from a binary operation.
    Binary,
    /// A column derived from a function.
    Function,
    /// A compound expression, potentially involving multiple operations or functions.
    Compound,
    /// A wildcard expression, representing multiple or all columns.
    Wildcard,
}

/// Represents the metadata of a parsed SQL query, including details about the tables and columns involved.
#[derive(Debug, Clone, PartialEq)]
pub struct UdiPgpStatment {
    /// Names of the tables involved in the query.
    pub tables: Vec<String>,
    /// Metadata about the columns involved in the query.
    pub columns: Vec<ColumnMetadata>,
    pub query: String,
    pub stmt: Statement,
    pub from_driver: bool
}
