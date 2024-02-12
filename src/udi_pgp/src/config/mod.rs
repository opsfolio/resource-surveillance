use config::Config;
use pgwire::error::{ErrorInfo, PgWireError, PgWireResult};
use regex::Regex;
use serde::de::{self, Error, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::{self, File};
use std::io::BufReader;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use tracing::error;

use crate::ssh::UdiPgpSshTarget;
use crate::{auth::Auth, error::UdiPgpResult, UdiPgpError, UdiPgpModes};

// pub mod manager;
mod nickel;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum SupplierType {
    Osquery,
    Git,
    Introspection,
}

impl Display for SupplierType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupplierType::Git => f.write_str("git"),
            SupplierType::Osquery => f.write_str("osquery"),
            SupplierType::Introspection => f.write_str("introspection"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Supplier {
    #[serde(rename = "type", deserialize_with = "deserialize_supplier_type")]
    pub supplier_type: SupplierType,
    pub mode: UdiPgpModes,
    #[serde(rename = "ssh-targets")]
    pub ssh_targets: Option<Vec<UdiPgpSshTarget>>,
    #[serde(
        rename = "atc-file-path",
        deserialize_with = "deserialize_atc_file_path",
        default
    )]
    pub atc_file_path: Option<String>,
    #[serde(default)]
    pub auth: Vec<Auth>,
}

fn deserialize_supplier_type<'de, D>(deserializer: D) -> Result<SupplierType, D::Error>
where
    D: Deserializer<'de>,
{
    struct SupplierTypeVisitor;

    impl<'de> Visitor<'de> for SupplierTypeVisitor {
        type Value = SupplierType;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a valid supplier type (git or osquery)")
        }

        fn visit_str<E>(self, value: &str) -> Result<SupplierType, E>
        where
            E: de::Error,
        {
            match value.to_lowercase().as_str() {
                "git" | "osquery" => Ok(match value {
                    "git" => SupplierType::Git,
                    "osquery" => SupplierType::Osquery,
                    _ => unreachable!(), // This should never happen
                }),
                _ => Err(de::Error::invalid_value(de::Unexpected::Str(value), &self)),
            }
        }
    }

    deserializer.deserialize_str(SupplierTypeVisitor)
}

fn deserialize_atc_file_path<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let path: Option<String> = Option::deserialize(deserializer)?;
    match &path {
        Some(p) => match fs::canonicalize(p) {
            Ok(resolved_path) => {
                if resolved_path.exists() {
                    Ok(Some(resolved_path.to_string_lossy().into_owned()))
                } else {
                    Err(serde::de::Error::custom(format!(
                            "Provided atc_file_path '{}' does not exist after resolution. Resolved path was: '{}'",
                            p,
                            resolved_path.to_string_lossy()
                        )))
                }
            }
            Err(_) => Err(serde::de::Error::custom(format!(
                "Failed to resolve the provided atc_file_path '{}'. Please ensure the path exists.",
                p
            ))),
        },
        None => Ok(None),
    }
}

impl Supplier {
    pub fn new(
        supplier_type: SupplierType,
        mode: UdiPgpModes,
        ssh_targets: Option<Vec<UdiPgpSshTarget>>,
        atc_file_path: Option<String>,
        auth: Vec<Auth>,
    ) -> Self {
        Supplier {
            supplier_type,
            mode,
            ssh_targets,
            atc_file_path,
            auth,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
// TODO add a watcher to the file if present
pub struct UdiPgpConfig {
    #[serde(default = "default_addr", deserialize_with = "deserialize_socket_addr")]
    addr: SocketAddr,
    pub metrics: Option<SocketAddr>,
    pub health: Option<SocketAddr>,
    #[serde(default)]
    pub suppliers: HashMap<String, Supplier>,
    #[serde(default = "default_verbose")]
    pub verbose: bool,
    #[serde(rename = "admin-state-fs-path", default = "default_admin_state_path")]
    pub admin_state_fs_path: String,
}

impl UdiPgpConfig {
    pub fn new(
        addr: SocketAddr,
        suppliers: HashMap<String, Supplier>,
        admin_db_file: &str,
    ) -> UdiPgpResult<UdiPgpConfig> {
        Ok(Config::builder()
            .build()?
            .try_deserialize::<UdiPgpConfig>()
            .map_err(UdiPgpError::ConfigBuilderError)?
            .with_addr(addr)
            .with_admin_db_file(admin_db_file)
            .with_suppliers(suppliers))
    }

    // TODO implement file watching with config crate
    // For ncl, write the config to a JSON file everytime it changes which will automatically trigger the watch
    pub fn try_from_file<P: AsRef<Path>>(path: P) -> UdiPgpResult<UdiPgpConfig> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| UdiPgpError::ConfigError("File has no extension".to_string()))?;

        match extension {
            "json" => Self::try_config_from_json(path.to_str().unwrap()),
            "ncl" => {
                let (config, _) = nickel::try_config_from_ncl(path.as_os_str())?;
                Ok(config)
            }
            other => Err(UdiPgpError::ConfigError(format!(
                "File extension not supported. Got {other:?}. Expected json or ncl"
            ))),
        }
    }

    fn try_config_from_json(path: &str) -> UdiPgpResult<UdiPgpConfig> {
        Config::builder()
            .add_source(config::File::with_name(path))
            .build()?
            .try_deserialize::<UdiPgpConfig>()
            .map_err(UdiPgpError::ConfigBuilderError)
    }

    pub fn with_addr(&mut self, addr: SocketAddr) -> Self {
        self.addr = addr;
        self.clone()
    }

    pub fn with_suppliers(&mut self, suppliers: HashMap<String, Supplier>) -> Self {
        self.suppliers = suppliers;
        self.clone()
    }

    pub fn with_admin_db_file(&mut self, db: &str) -> Self {
        self.admin_state_fs_path = db.to_string();
        self.clone()
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    pub fn host(&self) -> String {
        self.addr.ip().to_string()
    }

    pub fn supplier_auth(&self, name: &str, user: &str) -> PgWireResult<Option<Auth>> {
        if let Some(supplier) = self.suppliers.get(name) {
            Ok(supplier
                .auth
                .iter()
                .find(|auth| auth.user() == user)
                .cloned())
        } else {
            let error_message = format!("Could not find supplier in config file. Got: {}", name);
            error!("{}", error_message);
            Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                "FATAL".to_string(),
                "AUTH".to_string(),
                error_message,
            ))))
        }
    }

    // ====== UDI-PGP live config updates
    pub fn try_from_ncl_string(s: &str) -> UdiPgpResult<(UdiPgpConfig, PathBuf)> {
        nickel::try_config_from_ncl_string(s)
    }

    pub fn try_config_from_diagnostics(path: &PathBuf) -> UdiPgpResult<UdiPgpConfig> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let config = serde_json::from_reader(reader).map_err(|e| {
            UdiPgpError::ConfigError(format!("Failed to parse JSON from file. Erro: {}", e))
        })?;

        Ok(config)
    }

    /// Write the NCL string config to JSON file for diagnostics
    /// `core` designates if it is core configuration or supplier config
    pub fn diagnostics(s: &str, core: bool) -> UdiPgpResult<PathBuf> {
        nickel::ncl_to_json_file(s, core)
    }

    pub fn try_config_from_ncl_serve_supplier(
        s: &str,
    ) -> UdiPgpResult<(String, Supplier, PathBuf)> {
        let supplier_id = Self::get_supplier_id_from_serve_stmt(s)?;
        let (supplier, path) = nickel::try_supplier_from_ncl(s)?;
        Ok((supplier_id, supplier, path))
    }

    pub fn try_supplier_from_diagnostics(
        s: &str,
        path: &PathBuf,
    ) -> UdiPgpResult<(String, Supplier)> {
        let supplier_id = Self::get_supplier_id_from_serve_stmt(s)?;

        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let supplier: Supplier = serde_json::from_reader(reader).map_err(|err| {
            UdiPgpError::ConfigError(format!(
                "Failed to parse supplier: {} in file: {:#?}",
                err, path
            ))
        })?;
        Ok((supplier_id, supplier))
    }

    fn get_supplier_id_from_serve_stmt(s: &str) -> UdiPgpResult<String> {
        let re = Regex::new(r"let\s+([\w-]+)\s+=")
            .map_err(|err| UdiPgpError::ConfigError(err.to_string()))?;

        let remediation: &str = r"#`let supplier_name = { ... } in supplier_name`#";
        let caps = re.captures(s).ok_or(UdiPgpError::ConfigError(format!(
            "Expected: {remediation} got: {s}"
        )))?;

        let name = caps
            .get(1)
            .ok_or(UdiPgpError::ConfigError(format!(
                "Could not extract supplier name from config. Expected: {remediation}"
            )))?
            .as_str();
        Ok(name.to_string())
    }
}

fn default_addr() -> SocketAddr {
    "127.0.0.1:5432".to_socket_addrs().unwrap().next().unwrap()
}

fn deserialize_socket_addr<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<SocketAddr, D::Error> {
    let host_str: String = Deserialize::deserialize(deserializer)?;
    parse_socket_addr(&host_str).map_err(D::Error::custom)
}

fn parse_socket_addr(host_str: &str) -> UdiPgpResult<SocketAddr> {
    host_str.to_socket_addrs()?.next().ok_or_else(|| {
        UdiPgpError::ConfigError(format!("host '{host_str}' does not resolve to an IP"))
    })
}

fn default_verbose() -> bool {
    false
}

fn default_admin_state_path() -> String {
    "resource-surveillance-admin.sqlite.db".to_string()
}
