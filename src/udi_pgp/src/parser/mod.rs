use anyhow::anyhow;
use async_trait::async_trait;
use derive_new::new;
use pgwire::{
    api::{stmt::QueryParser, Type},
    error::{ErrorInfo, PgWireError, PgWireResult},
};
use sqlparser::{ast::Statement, dialect::PostgreSqlDialect, parser::Parser};

use stmt::UdiPgpStatment;

mod columns;
mod tables;
pub mod stmt;

static DRIVER_WORDS: [&str; 47] = [
    "show",
    "current_session",
    "session_user",
    "search_path",
    "current_schema()",
    // "set",
    "session",
    "committed",
    "datname",
    "pg_catalog.pg_database",
    "db.*",
    "select",
    "version()",
    "typecategory",
    "pg_catalog.pg_type",
    "c.relkind",
    "t.oid",
    "b.oid",
    "n.oid",
    "d.description",
    "select n.oid,n.*,d.description FROM pg_catalog.pg_ n",
    "pp.oid",
    "poid",
    "pp.proname",
    "timezone",
    "datestyle",
    "extra_float_digits",
    "start",
    "transaction",
    "repeatable",
    "nspname",
    "relname",
    "attname",
    "commit",
    "datestyle",
    "c.oid",
    "pg_get_expr",
    "application_name",
    "extra_float_digits",
    "application_name",
    "string_agg",
    "pg_get_keywords()",
    "array",
    "::text[]",
    "information_schema",
    "pg_catalog.pg_settings",
    "standard_conforming_strings",
    "client_min_messages",
];

#[derive(new, Debug, Default, Clone)]
// If I were to add datafusion, this is where it would come in.
// Notes: when it is time for udi specific queries, the queries could be deconstructed here
pub struct UdiPgpQueryParser;

impl UdiPgpQueryParser {
    pub fn parse(query: &str) -> PgWireResult<UdiPgpStatment> {
        let ast = Self::query_to_ast(query)
            .map_err(|err| PgWireError::StatementNotFound(err.to_string()))?;

        let stmt_query = match ast.clone() {
            Statement::Query(q) => q,
            other => {
                return Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                    "WARNING".to_string(),
                    "1111".to_string(),
                    format!("Expected SELECT, got: {}", other),
                ))))
            }
        };

        let tables = tables::get_table_names_from_query(*stmt_query.clone());
        let columns = columns::get_column_names_from_query(*stmt_query);

        Ok(UdiPgpStatment {
            tables,
            columns,
            query: query.to_string(),
            stmt: ast,
            from_driver: Self::check_if_query_is_from_driver(query),
        })
    }

    fn query_to_ast(query: &str) -> anyhow::Result<Statement> {
        let dialect = PostgreSqlDialect {};
        let res = Parser::parse_sql(&dialect, query);
        res.map_err(|err| anyhow!(err.to_string()))
            .and_then(|mut ast| match ast.len() {
                0 => Err(anyhow!("Empty input")),
                1 => Ok(ast.remove(0)),
                _ => Err(anyhow!("Expected only a single statement.")),
            })
    }

    // TODO: this is highly unoptimized. Consider cmparing Tokens from the AST or just traversing the AST and then compare
    fn check_if_query_is_from_driver(query: &str) -> bool {
        fn count_driver_words(query: &str) -> usize {
            DRIVER_WORDS
                .iter()
                .filter(|&&word| query.contains(word))
                .count()
        }

        let query_lower = query.to_lowercase();
        let query_trim = query_lower.trim();

        if query_trim.starts_with("close") {
            return true;
        }

        // Remove comments from query originating for vscode sql notebook
        let query = if query_trim.starts_with("--") {
            query_trim.split_once('\n').map_or(query_trim, |(_, q)| q)
        } else {
            query_trim
        };

        match query {
            q if q.contains("transaction_id") && q.starts_with("select") => false,
            q if q.contains("settings") && !q.contains("pg_catalog") => false,
            q if q.contains("start_time") && q.starts_with("select") => false,
            q if q.contains("commit") && !q.contains("string_agg") && q.starts_with("select") => {
                false
            }
            q if q.contains("select 1") => true,
            q if q.contains("select 1+1 as result") => true,
            _ => count_driver_words(query) >= 2,
        }
    }
}

#[async_trait]
impl QueryParser for UdiPgpQueryParser {
    type Statement = UdiPgpStatment;

    async fn parse_sql(&self, sql: &str, _types: &[Type]) -> PgWireResult<Self::Statement> {
        Self::parse(sql)
    }
}
