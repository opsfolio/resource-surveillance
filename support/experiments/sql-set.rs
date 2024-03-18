#!/usr/bin/env rust-script
//! This is a regular crate doc comment, but it also contains a partial
//! This is a regular crate doc comment, but it also contains a partial
//! Cargo manifest.  Note the use of a *fenced* code block, and the
//! `cargo` "language".
//!
//! ```cargo
//! [dependencies]
//! sqlparser = { version = "0.41.0", features = ["serde", "serde_json"] }
//! serde_json = { version = "*" }
//! ```
fn main() {
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

let sql = std::fs::read_to_string("./pgp.conf.sql")
        .unwrap_or_else(|_| panic!("Unable to read the conf file"));

let dialect = GenericDialect {}; // or AnsiDialect, or your own dialect ...

let ast = Parser::parse_sql(&dialect, &sql).unwrap();

 let serialized = serde_json::to_string_pretty(&ast).unwrap();
                    println!("Serialized as JSON:\n{serialized}");
}
