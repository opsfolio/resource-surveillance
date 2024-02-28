//! Default IMAP access by using app passwords

use std::{net::TcpStream, sync::Arc};

use rustls::RootCertStore;

use crate::ImapConfig;

pub fn retrieve_emails(config: &ImapConfig) -> anyhow::Result<()> {
  let mut root_store = RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let mut client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    client_config.key_log = Arc::new(rustls::KeyLogFile::new());

    let server_name = config.addr.clone().unwrap().clone().try_into()?;
    let mut conn = rustls::ClientConnection::new(Arc::new(client_config), server_name)?;
    let mut sock = TcpStream::connect(format!("{}:{}", config.addr.clone().unwrap(), config.port))?;
    let tls = rustls::Stream::new(&mut conn, &mut sock);

    let client = imap::Client::new(tls);

    let mut imap_session = client
        .login(&config.username.clone().unwrap(), &config.password.clone().unwrap())
        .map_err(|e| e.0)?;


  Ok(())
}