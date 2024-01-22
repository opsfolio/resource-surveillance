use anyhow::anyhow;
use async_trait::async_trait;
use derive_new::new;
use pgwire::{
    api::{stmt::QueryParser, Type},
    error::{ErrorInfo, PgWireError, PgWireResult},
};
use sqlparser::{ast::Statement, dialect::PostgreSqlDialect, parser::Parser};

use stmt::UdiPgpStatment;

use self::stmt::ColumnMetadata;

mod columns;
pub mod stmt;
mod tables;

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
    pub fn parse(query: &str, schema: bool) -> PgWireResult<UdiPgpStatment> {
        let ast = Self::query_to_ast(query).map_err(|err| {
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "FATAL".to_string(),
                "PARSER".to_string(),
                err.to_string(),
            )))
        })?;

        let (tables, columns) = if schema {
            Self::parse_create_table(&ast)?
        } else {
            Self::parse_query_statement(&ast)?
        };

        Ok(UdiPgpStatment {
            tables,
            columns,
            query: query.to_string(),
            stmt: ast,
            from_driver: Self::check_if_query_is_from_driver(query),
        })
    }

    fn parse_create_table(ast: &Statement) -> PgWireResult<(Vec<String>, Vec<ColumnMetadata>)> {
        match ast {
            Statement::CreateTable { name, columns, .. } => {
                let table_name = name
                    .0
                    .first()
                    .ok_or_else(|| {
                        PgWireError::UserError(Box::new(ErrorInfo::new(
                            "ERROR".to_string(),
                            "0001".to_string(),
                            "Missing table name in CREATE statement".to_string(),
                        )))
                    })?
                    .value
                    .clone();

                let cols = columns
                    .iter()
                    .map(ColumnMetadata::try_from)
                    .collect::<Result<Vec<_>, _>>()?;

                Ok((vec![table_name], cols))
            }
            other => Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                "WARNING".to_string(),
                "1111".to_string(),
                format!("Expected CREATE, got: {}", other),
            )))),
        }
    }

    fn parse_query_statement(ast: &Statement) -> PgWireResult<(Vec<String>, Vec<ColumnMetadata>)> {
        match ast {
            Statement::Query(q) => Ok((
                tables::get_table_names_from_query(q),
                columns::get_column_names_from_query(q),
            )),
            other => Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                "WARNING".to_string(),
                "1111".to_string(),
                format!("Expected SELECT, got: {}", other),
            )))),
        }
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
        Self::parse(sql, false)
    }
}
