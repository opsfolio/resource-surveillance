use config::Config;
use pgwire::error::{ErrorInfo, PgWireError, PgWireResult};
use regex::Regex;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Display;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::Path;
use tracing::error;

use crate::ssh::UdiPgpSshTarget;
use crate::{auth::Auth, error::UdiPgpResult, UdiPgpError, UdiPgpModes};

mod nickel;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum SupplierType {
    Osquery,
    Git,
}

impl Display for SupplierType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupplierType::Git => f.write_str("git"),
            SupplierType::Osquery => f.write_str("osquery"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Supplier {
    #[serde(rename = "type")]
    pub supplier_type: SupplierType,
    pub mode: UdiPgpModes,
    #[serde(rename = "ssh-targets")]
    pub ssh_targets: Option<Vec<UdiPgpSshTarget>>,
    #[serde(
        rename = "atc-file-path",
        deserialize_with = "deserialize_atc_file_path"
    )]
    pub atc_file_path: Option<String>,
    #[serde(default)]
    pub auth: Vec<Auth>,
}

fn deserialize_atc_file_path<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let path: Option<String> = Option::deserialize(deserializer)?;
    match path {
        Some(ref p) if Path::new(p).exists() => Ok(Some(p.clone())),
        None => Ok(None),
        _ => Err(serde::de::Error::custom(format!(
            "atc_file_path does not exist. Resolved path: {}",
            path.unwrap()
        ))),
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
}

impl UdiPgpConfig {
    pub fn new(
        addr: SocketAddr,
        suppliers: HashMap<String, Supplier>,
    ) -> UdiPgpResult<UdiPgpConfig> {
        Ok(Config::builder()
            .build()?
            .try_deserialize::<UdiPgpConfig>()
            .map_err(UdiPgpError::ConfigBuilderError)?
            .with_addr(addr)
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
            "ncl" => nickel::try_config_from_ncl(path.as_os_str()),
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
    pub fn try_from_ncl_string(s: &str) -> UdiPgpResult<UdiPgpConfig> {
        nickel::try_config_from_ncl_string(s)
    }

    pub fn try_config_from_ncl_serve_supplier(s: &str) -> UdiPgpResult<(String, Supplier)> {
        let supplier_id = Self::get_supplier_id_from_serve_stmt(s)?;
        Ok((supplier_id, nickel::try_supplier_from_ncl(s)?))
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
