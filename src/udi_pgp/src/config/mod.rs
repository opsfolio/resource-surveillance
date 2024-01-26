use config::Config;
use pgwire::error::{ErrorInfo, PgWireError, PgWireResult};
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::str::FromStr;
use tracing::error;

use crate::{auth::Auth, error::UdiPgpResult, UdiPgpError, UdiPgpModes};

#[derive(Debug, Clone, Deserialize)]
pub struct UdiPgpSshTarget {
    pub target: String,
    pub id: String,
    #[serde(rename = "  atc-file-path")]
    pub atc_file_path: Option<String>,
}

impl FromStr for UdiPgpSshTarget {
    type Err = UdiPgpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            println!();
            return Err(UdiPgpError::ConfigError(format!(
                "Target: {s} does not have exactly two parts. It has {} parts.",
                parts.len()
            )));
        }

        let target = parts[0];
        let id = parts[1];
        Ok(UdiPgpSshTarget {
            target: target.to_string(),
            id: id.to_string(),
            atc_file_path: None,
        })
    }
}

impl TryFrom<&String> for UdiPgpSshTarget {
    type Error = UdiPgpError;

    fn try_from(s: &std::string::String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            println!();
            return Err(UdiPgpError::ConfigError(format!(
                "Target: {s} does not have exactly two parts. It has {} parts.",
                parts.len()
            )));
        }

        let target = parts[0];
        let id = parts[1];
        Ok(UdiPgpSshTarget {
            target: target.to_string(),
            id: id.to_string(),
            atc_file_path: None,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Supplier {
    #[serde(default = "default_supplier_type", rename = "type")]
    pub supplier_type: String,
    #[serde(default = "default_mode")]
    pub mode: UdiPgpModes,
    pub ssh_targets: Option<Vec<UdiPgpSshTarget>>,
    #[serde(rename = "  atc-file-path")]
    pub atc_file_path: Option<String>,
    #[serde(default)]
    pub auth: Vec<Auth>,
}

impl Supplier {
    pub fn new(
        supplier_type: &str,
        mode: UdiPgpModes,
        ssh_targets: Option<Vec<UdiPgpSshTarget>>,
        atc_file_path: Option<String>,
        auth: Vec<Auth>,
    ) -> Self {
        Supplier {
            supplier_type: supplier_type.to_string(),
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
    pub metrics: Option<u16>,
    pub health: Option<u16>,
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

    //TODO use this for the ncl file straight up
    // pub fn load_from_json(json: &str) -> UdiPgpResult<UdiPgpConfig> {
    //     Ok(Config::builder()
    //         .add_source(FileSourceString::from(json))
    //         .build()?
    //         .try_deserialize::<UdiPgpConfig>()
    //         .map_err(UdiPgpError::ConfigBuilderError)?)
    // }

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

    pub fn execute(&self) -> anyhow::Result<()> {
        Ok(())
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

fn default_supplier_type() -> String {
    "osquery".to_string()
}

fn default_mode() -> UdiPgpModes {
    UdiPgpModes::Local
}
