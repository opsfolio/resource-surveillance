//! Each type of introspection table is modeled as an SqlSupplier since they all
//! represent a distinct resource which is suppliers, core configuration and the logs

pub mod core_table;
pub mod logs_table;
pub mod suppliers_table;
