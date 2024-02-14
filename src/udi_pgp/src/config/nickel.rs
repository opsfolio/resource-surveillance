use nickel_lang_core::{
    error::{report::ErrorFormat, Error as NickelError},
    eval::cache::CacheImpl,
    program::Program,
    serialize::{self, ExportFormat},
    term::RichTerm,
};
use serde_json::Value;
use std::{
    ffi::OsString,
    fs::File,
    io::{BufReader, Cursor, Write},
    path::PathBuf,
};
use tempfile::NamedTempFile;
use tracing::error;

use crate::error::{UdiPgpError, UdiPgpResult};

use super::{Supplier, UdiPgpConfig};

/// Write the NCL string config to JSON file for diagnostics
pub fn ncl_to_json_file(s: &str, core: bool) -> UdiPgpResult<PathBuf> {
    let src = Cursor::new(s);
    let source_name = if core { "<config>" } else { "<supplier>" };
    let mut program =
        Program::new_from_source(src, source_name, std::io::sink()).map_err(|err| {
            error!("{}", err);
            UdiPgpError::ConfigError(err.to_string())
        })?;

    let json = export(&mut program, ExportFormat::Json).map_err(|err| {
        program.report(err, ErrorFormat::Text);
        UdiPgpError::ConfigError("Failed to export configuration".to_string())
    })?;

    write_and_persist_temp(&json)
}

pub fn try_supplier_from_ncl(s: &str) -> UdiPgpResult<(Supplier, PathBuf)> {
    let src = Cursor::new(s);
    let mut program =
        Program::new_from_source(src, "<supplier>", std::io::sink()).map_err(|err| {
            error!("{}", err);
            UdiPgpError::ConfigError(err.to_string())
        })?;

    let json = export(&mut program, ExportFormat::Json).map_err(|err| {
        program.report(err, ErrorFormat::Text);
        UdiPgpError::ConfigError("Failed to export configuration".to_string())
    })?;

    let path = write_and_persist_temp(&json)?;
    let file = File::open(&path)?;
    let reader = BufReader::new(file);

    let supplier: Supplier = serde_json::from_reader(reader).map_err(|err| {
        UdiPgpError::ConfigError(format!(
            "Failed to parse supplier: {} in file: {:#?}",
            err, path
        ))
    })?;

    Ok((supplier, path))
}

pub fn try_config_from_ncl(path: impl Into<OsString>) -> UdiPgpResult<(UdiPgpConfig, PathBuf)> {
    let mut program = Program::new_from_file(path, std::io::stderr()).map_err(|err| {
        error!("{}", err);
        UdiPgpError::ConfigError(err.to_string())
    })?;

    let config = export(&mut program, ExportFormat::Json).map_err(|err| {
        program.report(err, ErrorFormat::Text);
        UdiPgpError::ConfigError("Failed to export configuration".to_string())
    })?;

    config_from_json(&config, true)
}

pub fn try_config_from_ncl_string(s: &str) -> UdiPgpResult<(UdiPgpConfig, PathBuf)> {
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

    config_from_json(&config, false)
}

fn export(program: &mut Program<CacheImpl>, format: ExportFormat) -> Result<String, NickelError> {
    let rt = program.eval_full_for_export().map(RichTerm::from)?;
    serialize::validate(format, &rt)?;
    Ok(serialize::to_string(format, &rt)?)
}

fn config_from_json(json: &str, config_key: bool) -> UdiPgpResult<(UdiPgpConfig, PathBuf)> {
    let path = write_and_persist_temp(json)?;
    let file = File::open(&path)?;
    let reader = BufReader::new(file);

    let value: Value = serde_json::from_reader(reader).map_err(|e| {
        UdiPgpError::ConfigError(format!("Failed to parse JSON from file. Erro: {}", e))
    })?;

    let config: UdiPgpConfig = if config_key {
        let config = value
            .get("config")
            .ok_or_else(|| UdiPgpError::ConfigError("Missing 'config' key in JSON".to_string()))?
            .clone();

        serde_json::from_value(config).map_err(|err| {
            error!("{}", err);
            UdiPgpError::ConfigError("Failed to deserialize 'config'".to_string())
        })?
    } else {
        serde_json::from_str(json)
            .map_err(|_| UdiPgpError::ConfigError("Failed to parse JSON".to_string()))?
    };

    Ok((config, path))
}

fn write_and_persist_temp(s: &str) -> UdiPgpResult<PathBuf> {
    let mut temp_file = NamedTempFile::new()?;
    let temp_file_path = temp_file.path().with_extension("json");

    temp_file.write_all(s.as_bytes())?;

    let _ = temp_file.persist(&temp_file_path).map_err(|err| {
        error!("{}", err);
        UdiPgpError::ConfigError(format!(
            "Failed to create temp config file at: {:#?}",
            temp_file_path
        ))
    });
    Ok(temp_file_path)
}
