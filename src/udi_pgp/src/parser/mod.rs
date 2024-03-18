use std::str::FromStr;

use anyhow::anyhow;
use async_trait::async_trait;
use derive_new::new;
use pgwire::{
    api::{stmt::QueryParser, Type},
    error::{ErrorInfo, PgWireError, PgWireResult},
};
use regex::Regex;
use sqlparser::{ast::Statement, dialect::PostgreSqlDialect, parser::Parser};

use stmt::UdiPgpStatment;

use crate::{error::UdiPgpResult, introspection::IntrospectionTable};

use self::stmt::{ColumnMetadata, StmtType};

mod columns;
pub mod stmt;
mod tables;

static DRIVER_WORDS: [&str; 50] = [
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
    "pg_constraint",
    "r.contype",
    "pg_get_constraintdef",
];

#[derive(new, Debug, Default, Clone)]
// If I were to add datafusion, this is where it would come in.
// Notes: when it is time for udi specific queries, the queries could be deconstructed here
pub struct UdiPgpQueryParser;

impl UdiPgpQueryParser {
    pub fn parse(query: &str, schema: bool) -> PgWireResult<UdiPgpStatment> {
        let query = Self::remove_sql_comments(query)?;
        let ast = Self::parse_query_to_ast(&query)?;
        let config_query = Self::query_is_udi_configuration(&ast);
        let (tables, columns) = Self::determine_tables_and_columns(schema, config_query, &ast)?;
        let introspection_query = Self::is_introspection_query(&tables);

        Ok(UdiPgpStatment {
            tables,
            columns,
            query: query.to_string(),
            stmt: ast,
            stmt_type: Self::determine_statement_type(&query, config_query, introspection_query),
        })
    }

    fn remove_sql_comments(query: &str) -> UdiPgpResult<String> {
        let re_single_line = Regex::new(r"--[^\n]*").unwrap();
        let re_multi_line = Regex::new(r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/").unwrap();

        // First, remove multi-line comments
        let no_multi_line_comments = re_multi_line.replace_all(query, "");

        // Then, remove single-line comments
        let no_comments = re_single_line.replace_all(&no_multi_line_comments, "");

        Ok(no_comments.into_owned())
    }

    fn parse_query_to_ast(query: &str) -> PgWireResult<Statement> {
        Self::query_to_ast(query).map_err(|err| {
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "FATAL".to_string(),
                "PARSER".to_string(),
                err.to_string(),
            )))
        })
    }

    fn determine_tables_and_columns(
        schema: bool,
        config_query: bool,
        ast: &Statement,
    ) -> PgWireResult<(Vec<String>, Vec<ColumnMetadata>)> {
        if schema {
            Self::parse_create_table(ast)
        } else if config_query {
            Ok((vec![], vec![]))
        } else {
            Self::parse_query_statement(ast)
        }
    }

    fn is_introspection_query(tables: &[String]) -> bool {
        tables
            .iter()
            .any(|t| IntrospectionTable::from_str(t.as_str()).is_ok())
    }

    fn determine_statement_type(
        query: &str,
        config_query: bool,
        introspection_query: bool,
    ) -> StmtType {
        if Self::check_if_query_is_from_driver(query) {
            StmtType::Driver
        } else if config_query {
            StmtType::Config
        } else if introspection_query {
            StmtType::Introspection
        } else {
            StmtType::Supplier
        }
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

    /// This checks for configuration queries. e.e
    /// SET udi_pgp_serve_ncl = '' or
    /// SET udi_pgp_serve_json = '' or
    /// STE udi_pgp_serve_ncl_uri or
    /// STE udi_pgp_serve_json_uri
    fn query_is_udi_configuration(ast: &Statement) -> bool {
        match ast {
            Statement::SetVariable { variable, .. } => {
                let name = &variable.0.first().unwrap().value;
                let targets = [
                    "udi_pgp_serve_ncl",
                    "udi_pgp_serve_json",
                    "udi_pgp_serve_ncl_uri",
                    "udi_pgp_serve_uri",
                ];

                targets.iter().any(|&target| name.contains(target))
            }
            _ => false,
        }
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
