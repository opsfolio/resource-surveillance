use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;
use udi_pgp_imap::{ImapConfig, MsftAuthServerConfig, MsftConfig};
const DEFAULT_STATEDB_FS_PATH: &str = "resource-surveillance.sqlite.db";

#[derive(Debug, Serialize, Clone, ValueEnum, Default)]
pub enum MsftAuthMethod {
    AuthCode,
    #[default]
    DeviceCode,
}

#[derive(Debug, Serialize, Args, Clone)]
pub struct MsftSericeArgs {
    /// Client ID of the application from MSFT Azure App Directory
    #[arg(short = 'i', long)]
    pub client_id: String,
    /// Client Secret of the application from MSFT Azure App Directory
    #[arg(short = 's', long)]
    pub client_secret: String,
    /// The mode to generate an access_token. Default is 'DeviceCode'.
    #[arg(short = 'm', long)]
    pub mode: MsftAuthMethod,
    /// Address to start the authentication server on, when using the `auth_code` mode for token generation.
    #[arg(short = 'a', long, default_value = "http://127.0.0.1:8000")]
    pub addr: Option<String>,
    /// Redirect URL. Base redirect URL path. It gets concatenated with the server address to form the full redirect url,
    /// when using the `auth_code` mode for token generation.
    #[arg(short = 'r', long, default_value = "/redirect")]
    pub redirect_uri: Option<String>,
}

/// Email services that require oauth or a more complicated workflow
#[derive(Debug, Serialize, Subcommand, Clone)]
pub enum ServiceCommands {
    Msft(MsftSericeArgs),
}

#[derive(Debug, Serialize, Clone, ValueEnum, Default)]
pub enum ImapMessageStatus {
    #[default]
    Unread,
    Read,
    Starred,
}

/// Ingest content from email boxes
#[derive(Debug, Serialize, Args, Clone)]
pub struct IngestImapArgs {
    /// target SQLite database
    #[arg(short='d', long, default_value = DEFAULT_STATEDB_FS_PATH, default_missing_value = "always", env="SURVEILR_STATEDB_FS_PATH")]
    pub state_db_fs_path: String,

    /// one or more globs to match as SQL files and batch execute them in alpha order
    #[arg(short = 'I', long)]
    pub state_db_init_sql: Vec<String>,

    /// email address
    #[arg(short, long)]
    pub username: Option<String>,

    /// password to the email. mainly an app password.
    /// See the documentation on how to create an app password
    #[arg(short, long)]
    pub password: Option<String>,

    /// IMAP server address. e.g imap.gmail.com or outlook.office365.com
    #[arg(short = 'a', long)]
    pub server_addr: Option<String>,

    /// IMAP server port.
    #[arg(long, default_value = "993")]
    pub port: u16,

    /// Mailboxes to read from. i.e folders.
    #[arg(short, long, default_value = "INBOX")]
    pub folders: Vec<String>,

    /// Status of the messages to be ingested.
    #[arg(short, long, default_value = "unread")]
    pub status: Vec<ImapMessageStatus>,

    /// Maximum number of messages to be ingested.
    #[arg(short, long, default_value = "100")]
    pub max_no_messages: u64,

    /// Extract Attachments
    #[arg(short, long, default_value = "true")]
    pub extract_attachments: bool,

    /// Command line configuration for services that need extra authenctication to access emails.
    #[command(subcommand)]
    pub command: Option<ServiceCommands>,
}

impl From<IngestImapArgs> for ImapConfig {
    fn from(value: IngestImapArgs) -> Self {
        ImapConfig {
            username: value.username,
            password: value.password,
            addr: value.server_addr,
            port: value.port,
            folders: value.folders,
            max_no_messages: value.max_no_messages,
            extract_attachments: value.extract_attachments,
            msft: {
                if let Some(service_cmds) = value.command {
                    match service_cmds {
                        ServiceCommands::Msft(config) => {
                            let (server, redirect_uri) = match (config.addr, config.redirect_uri) {
                                (Some(a), Some(r)) => {
                                    let full_redirect_url = format!("{a}{r}");
                                    let server_config = MsftAuthServerConfig {
                                        addr: a,
                                        base_url: r,
                                    };
                                    (Some(server_config), Some(full_redirect_url))
                                }
                                _ => (None, None),
                            };

                            let msft_config = MsftConfig {
                                client_id: config.client_id,
                                client_secret: config.client_secret,
                                redirect_uri,
                                mode: {
                                    match config.mode {
                                        MsftAuthMethod::AuthCode => {
                                            udi_pgp_imap::TokenGenerationMethod::AuthCode
                                        }
                                        MsftAuthMethod::DeviceCode => {
                                            udi_pgp_imap::TokenGenerationMethod::DeviceCode
                                        }
                                    }
                                },
                                auth_server: server,
                            };
                            Some(msft_config)
                        }
                    }
                } else {
                    None
                }
            },
        }
    }
}
