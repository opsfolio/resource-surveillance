use std::net::SocketAddr;

use derive_new::new;

use crate::auth::Auth;

#[derive(Debug, Clone, new)]
pub struct UdiPgpConfig {
    addr: SocketAddr,
    auth: Auth,
}

impl UdiPgpConfig {
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
