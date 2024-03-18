use serde::{Deserialize, Serialize};

//all the driver queries for extended queries or otherwise are listed here
pub const SELECT_CURRENT_SCHEMA: &str = "SELECT current_schema(),session_user";
pub const SELECT_PG_CATALOG: &str =
    "SELECT db.oid,db.* FROM pg_catalog.pg_database db WHERE datname=$1";
pub const SELECT_VERSION_DBEAVER: &str = "SELECT version()";
pub const SELECT_VERSION: &str = "select version();";
pub const SELECT_TYPCATEGORY: &str =
    "SELECT typcategory FROM pg_catalog.pg_type WHERE 1<>1 LIMIT 1";
pub const SELECT_PG_SETTINGS: &str = "select * from pg_catalog.pg_settings";

pub const SET_SEARCH_PATH: &str = "SET search_path = pg_catalog";
pub const SET_TIME_ZONE: &str = "SET timezone = 'UTC'";
pub const SET_DATE_STYLE: &str = "SET datestyle = ISO";
pub const SET_EXTRA_FLOAT_DIGITS: &str = "SET extra_float_digits = 2";
pub const START_TRANSACTION: &str = "START TRANSACTION ISOLATION LEVEL REPEATABLE READ";
pub const COMMIT_TRANSACTION: &str = "COMMIT TRANSACTION";
pub const CLOSE_CURSOR: &str = "CLOSE c1";
pub const SHOW_DATE_STYLE: &str = "SHOW DateStyle;";
pub const SHOW_SEARCH_PATH: &str = "SHOW search_path";

pub const SELECT_VERSION_RESPONSE: &str = "PostgreSQL 14.7 (Ubuntu 14.7-1.pgdg20.04+1) on x86_64-pc-linux-gnu, compiled by gcc (Ubuntu 9.4.0-1ubuntu1~20.04.1) 9.4.0, 64-bit";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(from = "String")]
enum Setting {
    Float(f64),
    Str(String),
    Int(i64),
}
use std::str;

impl From<String> for Setting {
    fn from(value: String) -> Self {
        if let Ok(i) = value.parse() {
            Setting::Int(i)
        } else if let Ok(i) = value.parse() {
            Setting::Float(i)
        } else {
            Setting::Str(value.to_string())
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PgSettings {
    name: String,
    setting: Setting,
    unit: Option<String>,
    category: Option<String>,
    short_desc: String,
    extra_desc: Option<String>,
    context: Option<String>,
    vartype: Option<String>,
    source: Option<String>,
    min_val: Option<String>,
    max_val: Option<String>,
    enumvals: Option<String>,
    boot_val: Option<String>,
    reset_val: Option<String>,
    sourcefile: Option<String>,
    sourceline: Option<i64>,
    pending_restart: bool,
}

pub mod response;
