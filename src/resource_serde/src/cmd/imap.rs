use clap::{Args, Subcommand, ValueEnum};
use resource_imap::{ImapConfig, Microsoft365AuthServerConfig, Microsoft365Config};
use serde::Serialize;
const DEFAULT_STATEDB_FS_PATH: &str = "resource-surveillance.sqlite.db";

#[derive(Debug, Serialize, Clone, ValueEnum, Default)]
pub enum Microsoft365AuthMethod {
    AuthCode,
    #[default]
    DeviceCode,
    // ClientCredential,
}

#[derive(Debug, Serialize, Args, Clone)]
pub struct Microsoft365ServiceArgs {
    /// Client ID of the application from MSFT Azure App Directory
    #[arg(short = 'i', long, env = "MICROSOFT_365_CLIENT_ID")]
    pub client_id: String,
    /// Client Secret of the application from MSFT Azure App Directory
    #[arg(short = 's', long, env = "MICROSOFT_365_CLIENT_SECRET")]
    pub client_secret: String,
    /// The mode to generate an access_token. Default is 'DeviceCode'.
    #[arg(short = 'm', long)]
    pub mode: Microsoft365AuthMethod,
    /// Address to start the authentication server on, when using the `auth_code` mode for token generation.
    #[arg(
        short = 'a',
        long,
        default_value = "http://127.0.0.1:8000",
        env = "MICROSOFT_365_CLIENT_REDIRECT_URI"
    )]
    pub addr: Option<String>,
    /// Redirect URL. Base redirect URL path. It gets concatenated with the server address to form the full redirect url,
    /// when using the `auth_code` mode for token generation.
    #[arg(short = 'r', long, default_value = "/redirect")]
    pub redirect_uri: Option<String>,
    /// Port to bind the server to
    #[arg(short = 'p', long, default_value = "8000")]
    pub port: u16,
}

/// Email services that require oauth or a more complicated workflow
#[derive(Debug, Serialize, Subcommand, Clone)]
pub enum ServiceCommands {
    /// Microsoft 365 Credentials
    #[clap(name = "microsoft-365")]
    Microsoft365(Microsoft365ServiceArgs),
}

#[derive(Debug, Serialize, Clone, ValueEnum, Default)]
pub enum ImapMessageStatus {
    #[default]
    All,
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

    /// Mailboxes to read from. i.e folders. Takes a regular expression matching the folder names.
    /// The default is a "*" which means all folders.
    #[arg(short, long, default_value = "*")]
    pub folder: String,

    /// Status of the messages to be ingested.
    #[arg(short, long, default_value = "unread")]
    pub status: Vec<ImapMessageStatus>,

    /// Maximum number of messages to be ingested.
    #[arg(short, long, default_value = "1000")]
    pub batch_size: u64,

    /// Extract Attachments
    #[arg(short, long, default_value = "true")]
    pub extract_attachments: bool,

    /// Display progress animation for emails downloading and processing
    #[arg(long, default_value = "false")]
    pub progress: bool,

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
            folder: value.folder,
            mailboxes: vec![],
            batch_size: value.batch_size,
            extract_attachments: value.extract_attachments,
            progress: value.progress,
            microsoft365: {
                if let Some(service_cmds) = value.command {
                    match service_cmds {
                        ServiceCommands::Microsoft365(config) => {
                            let (server, redirect_uri) = match (config.addr, config.redirect_uri) {
                                (Some(a), Some(r)) => {
                                    let full_redirect_url = format!("{a}{r}");
                                    let server_config = Microsoft365AuthServerConfig {
                                        addr: a,
                                        base_url: r,
                                        port: config.port,
                                    };
                                    (Some(server_config), Some(full_redirect_url))
                                }
                                _ => (None, None),
                            };

                            let msft_config = Microsoft365Config {
                                client_id: config.client_id,
                                client_secret: config.client_secret,
                                redirect_uri,
                                mode: {
                                    match config.mode {
                                        Microsoft365AuthMethod::AuthCode => {
                                            resource_imap::TokenGenerationMethod::AuthCode
                                        }
                                        Microsoft365AuthMethod::DeviceCode => {
                                            resource_imap::TokenGenerationMethod::DeviceCode
                                        }
                                        // Microsoft365AuthMethod::ClientCredential => {
                                        //     resource_imap::TokenGenerationMethod::ClientCredential
                                        // }
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
