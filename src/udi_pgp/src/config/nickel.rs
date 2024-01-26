use nickel_lang_core::{
    error::{report::ErrorFormat, Error as NickelError},
    eval::cache::CacheImpl,
    program::Program,
    serialize::{self, ExportFormat},
    term::RichTerm,
};
use serde_json::Value;
use std::{ffi::OsString, io::Cursor};
use tracing::error;

use crate::error::{UdiPgpError, UdiPgpResult};

use super::UdiPgpConfig;

pub fn try_config_from_ncl(path: impl Into<OsString>) -> UdiPgpResult<UdiPgpConfig> {
    let mut program = Program::new_from_file(path, std::io::stderr()).map_err(|err| {
        error!("{}", err);
        UdiPgpError::ConfigError(err.to_string())
    })?;

    let config = export(&mut program, ExportFormat::Json).map_err(|err| {
        program.report(err, ErrorFormat::Text);
        UdiPgpError::ConfigError("Failed to export configuration".to_string())
    })?;

    config_from_json(&config)
}

pub fn try_config_from_ncl_string(s: &str) -> UdiPgpResult<UdiPgpConfig> {
    let src = Cursor::new(s);
    let mut program =
        Program::new_from_source(src, "<config>", std::io::sink()).map_err(|err| {
            error!("{}", err);
            UdiPgpError::ConfigError(err.to_string())
        })?;

    let config = export(&mut program, ExportFormat::Json).map_err(|err| {
        program.report(err, ErrorFormat::Text);
        UdiPgpError::ConfigError("Failed to export configuration".to_string())
    })?;

    config_from_json(&config)
}

fn export(program: &mut Program<CacheImpl>, format: ExportFormat) -> Result<String, NickelError> {
    let rt = program.eval_full_for_export().map(RichTerm::from)?;
    serialize::validate(format, &rt)?;
    Ok(serialize::to_string(format, &rt)?)
}

fn config_from_json(json: &str) -> UdiPgpResult<UdiPgpConfig> {
    let value: Value = serde_json::from_str(json)
        .map_err(|_| UdiPgpError::ConfigError("Failed to parse JSON".to_string()))?;

    let config = value
        .get("config")
        .ok_or_else(|| UdiPgpError::ConfigError("Missing 'config' key in JSON".to_string()))?
        .clone();

    serde_json::from_value(config).map_err(|err| {
        error!("{}", err);
        UdiPgpError::ConfigError("Failed to deserialize 'config'".to_string())
    })
}
