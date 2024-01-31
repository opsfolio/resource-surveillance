use std::{fmt::Display, str::FromStr};

use serde::Deserialize;

use crate::error::UdiPgpError;

pub mod key;
pub mod session;
#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
pub struct UdiPgpSshTarget {
    pub host: String,
    pub port: Option<u16>,
    pub user: String,
    pub id: String,
    #[serde(rename = "atc-file-path")]
    pub atc_file_path: Option<String>,
}

impl TryFrom<&String> for UdiPgpSshTarget {
    type Error = UdiPgpError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        UdiPgpSshTarget::from_str(value)
    }
}

impl FromStr for UdiPgpSshTarget {
    type Err = UdiPgpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use UdiPgpError::SshConnectionParseError;

        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err(SshConnectionParseError(format!(
                "Target: {s} does not have exactly two parts. It has {} parts.",
                parts.len()
            )));
        }

        let s = format!("ssh://{}", parts[0]);
        let id = parts[1];

        if !s.starts_with("ssh://") {
            return Err(SshConnectionParseError(format!(
                "connection string should start with `ssh://`: {}",
                s
            )));
        }

        let s = &s["ssh://".len()..];

        let (user, rest) = s.split_once('@').ok_or_else(|| {
            SshConnectionParseError(format!(
                "connection string should have the format `ssh://user@address`: {}",
                s
            ))
        })?;

        if user.is_empty() {
            return Err(SshConnectionParseError(format!(
                "user cannot be empty: {}",
                s
            )));
        }

        let (host, port_str) = rest.rsplit_once(':').unwrap_or((rest, ""));
        let port = if !port_str.is_empty() {
            Some(port_str.parse().map_err(|_| {
                SshConnectionParseError(format!("port should be a valid number: {}", port_str))
            })?)
        } else {
            None
        };

        if host.is_empty() {
            return Err(SshConnectionParseError(format!(
                "host cannot be empty: {}",
                s
            )));
        }

        Ok(Self {
            host: host.to_owned(),
            port,
            user: user.to_owned(),
            id: id.to_owned(),
            atc_file_path: None,
        })
    }
}

#[derive(Debug)]
pub enum SshConnection {
    ConnectionString(String),
    Parameters(UdiPgpSshTarget),
}

impl Display for SshConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionString(s) => write!(f, "{s}"),
            Self::Parameters(UdiPgpSshTarget {
                host, port, user, ..
            }) => {
                write!(f, "ssh://{user}@{host}")?;
                if let Some(port) = port {
                    write!(f, ":{port}")?;
                }
                Ok(())
            }
        }
    }
}

impl SshConnection {
    pub fn connection_string(&self) -> String {
        format!("{self}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_connection_string() {
        let conn_str = SshConnection::ConnectionString("ssh://prod@127.0.0.1:5432".to_string())
            .connection_string();
        assert_eq!(&conn_str, "ssh://prod@127.0.0.1:5432");

        let conn_str = SshConnection::Parameters(UdiPgpSshTarget {
            host: "127.0.0.1".to_string(),
            port: Some(5432),
            user: "prod".to_string(),
            id: "prod".to_string(),
            atc_file_path: None,
        });
        let conn_str = conn_str.connection_string();
        assert_eq!(&conn_str, "ssh://prod@127.0.0.1:5432");

        // Missing port.
        let conn_str = SshConnection::Parameters(UdiPgpSshTarget {
            host: "127.0.0.1".to_string(),
            port: None,
            user: "prod".to_string(),
            id: "prod".to_string(),
            atc_file_path: None,
        });
        let conn_str = conn_str.connection_string();
        assert_eq!(&conn_str, "ssh://prod@127.0.0.1");
    }

    #[test]
    fn parse_connection_string() {
        // Valid
        let test_cases = vec![
            (
                "ssh://user@host.com",
                UdiPgpSshTarget {
                    host: "host.com".to_string(),
                    port: None,
                    user: "user".to_string(),
                    id: "prod".to_string(),
                    atc_file_path: None,
                },
            ),
            (
                "ssh://user@host.com:1234",
                UdiPgpSshTarget {
                    host: "host.com".to_string(),
                    port: Some(1234),
                    user: "user".to_string(),
                    id: "prod".to_string(),
                    atc_file_path: None,
                },
            ),
            (
                "ssh://user@127.0.0.1:1234",
                UdiPgpSshTarget {
                    host: "127.0.0.1".to_string(),
                    port: Some(1234),
                    user: "user".to_string(),
                    id: "prod".to_string(),
                    atc_file_path: None,
                },
            ),
        ];
        for (s, v) in test_cases {
            let s: UdiPgpSshTarget = s.parse().unwrap();
            assert_eq!(s, v);
        }

        // Invalid
        let test_cases = vec![
            "random string",
            "user@host.com",          // doesn't start with `ssh://`
            "ssh://user_at_host.com", // missing `@`
            "ssh://@host.com",        // empty user
            "ssh://user@",            // empty address
            "ssh://user@:1234",       // empty host
            "ssh://host.com:abc",     // invalid port
        ];
        for s in test_cases {
            s.parse::<UdiPgpSshTarget>()
                .expect_err("invalid ssh connection string should error");
        }
    }
}
