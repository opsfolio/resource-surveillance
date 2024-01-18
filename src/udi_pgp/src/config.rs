use std::net::SocketAddr;

use crate::{auth::Auth, UdiPgpModes};
use resource_serde::cmd::udi_pgp::{OsqueryCommands, UdiPgpArgs, UdiPgpCommands};

#[derive(Debug, Clone)]
pub struct UdiPgpConfig {
    pub mode: UdiPgpModes,
    addr: SocketAddr,
    auth: Auth,
}

impl UdiPgpConfig {
    pub fn new(args: &UdiPgpArgs) -> Self {
        let mode = match &args.command {
            UdiPgpCommands::Osquery(args) => match args.command {
                OsqueryCommands::Local => UdiPgpModes::Local,
                OsqueryCommands::Remote => UdiPgpModes::Remote,
            },
        };

        let auth = Auth::new(&args.username, &args.password);
        UdiPgpConfig {
            mode,
            addr: args.addr,
            auth,
        }
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

    pub fn auth(&self) -> &Auth {
        &self.auth
    }

    pub fn execute(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl From<&UdiPgpArgs> for UdiPgpConfig {
    fn from(value: &UdiPgpArgs) -> Self {
        UdiPgpConfig::new(value)
    }
}
