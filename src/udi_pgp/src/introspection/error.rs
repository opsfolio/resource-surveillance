use thiserror::Error;

#[derive(Error, Debug)]
pub enum IntrospectionError {
    #[error("{0}")]
    TableError(String),
}
