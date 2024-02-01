use pgwire::error::{ErrorInfo, PgWireError};
use std::{
    fmt::Display,
    io::{Error as IOError, ErrorKind},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UdiPgpError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    PgWireError(#[from] PgWireError),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error("Error: {1} from {0} supplier: {2}")]
    SupplierError(String, UdiPgpErrorSeverity, String),
    #[error("Failed to convert {0} to type: {1}")]
    TypeConversionError(String, String),
    /// The table name and the error
    #[error("Failed to generate schema for: {0}. Error: {1}")]
    SchemaError(String, String),
    #[error("{0}")]
    QueryExecutionError(String),
    #[error("Invalid SSH connection string: {0}")]
    SshConnectionParseError(String),
    #[error(transparent)]
    SshTunnelError(#[from] crate::ssh::session::SshTunnelError),
    #[error(transparent)]
    SshKeyError(#[from] crate::ssh::key::SshKeyError),
    #[error("{0}")]
    ConfigError(String),
    #[error(transparent)]
    ConfigBuilderError(#[from] config::ConfigError),
    #[error(transparent)]
    IntrospectionError(#[from] crate::introspection::IntrospectionError),
}

#[derive(Debug, Clone)]
pub enum UdiPgpErrorSeverity {
    Fatal,
    Warning,
    Message,
}

impl Display for UdiPgpErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UdiPgpErrorSeverity::Fatal => f.write_str("FATAL"),
            UdiPgpErrorSeverity::Message => f.write_str("MESSAGE"),
            UdiPgpErrorSeverity::Warning => f.write_str("WARNING"),
        }
    }
}

impl From<UdiPgpError> for IOError {
    fn from(e: UdiPgpError) -> Self {
        IOError::new(ErrorKind::Other, e)
    }
}

impl From<UdiPgpError> for PgWireError {
    fn from(value: UdiPgpError) -> Self {
        match value {
            UdiPgpError::SupplierError(name, severity, msg) => PgWireError::UserError(Box::new(
                ErrorInfo::new(severity.to_string(), format!("Supplier-{}", name), msg),
            )),
            UdiPgpError::PgWireError(err) => err,
            other => PgWireError::UserError(Box::new(ErrorInfo::new(
                "ERROR".to_string(),
                "1111".to_string(),
                other.to_string(),
            ))),
        }
    }
}

pub type UdiPgpResult<T> = Result<T, UdiPgpError>;
