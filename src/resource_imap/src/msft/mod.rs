use anyhow::anyhow;
use async_trait::async_trait;
use graph_rs_sdk::oauth::{AccessToken, OAuth};
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tracing::warn;

use crate::{Folder, ImapConfig, ImapResource};

use self::emails::MsftGraphApiEmail;

mod auth_code;
// mod client_credential;
mod device_code;
mod emails;

/// The method for retrieving the access token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenGenerationMethod {
    /// Use the device code, that is, authenticate through another device.
    /// https://learn.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-device-code
    DeviceCode,
    /// Utilize a redirect url to get the access token
    AuthCode,
    // ClientCredential,
}

impl Display for TokenGenerationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenGenerationMethod::AuthCode => f.write_str("auth code grant"),
            TokenGenerationMethod::DeviceCode => f.write_str("device code"),
            // TokenGenerationMethod::ClientCredential => f.write_str("client credentials"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Microsoft365AuthServerConfig {
    /// Address to bind the server to
    pub addr: String,
    /// Base redirect url. Defaults to `/redirect`
    pub base_url: String,
    /// Port to start the server on
    pub port: u16,
}

/// Using the Graph API
#[derive(Debug, Clone)]
pub struct MicrosoftImapResource {
    /// Client ID of the application from MSFT Azure App Directory
    client_id: String,
    /// Client Secret of the application from MSFT Azure App Directory
    client_secret: String,
    /// An optional redirect URL for `access_token` generation when using the AuthCode mode
    redirect_uri: Option<String>,
    /// The mode to generate an access_token. Default is 'DeviceCode'.
    mode: TokenGenerationMethod,
    /// Address to start the authentication server on. Used by the redirect_uri
    auth_server: Option<Microsoft365AuthServerConfig>,
    /// Access Token
    access_token: Option<AccessToken>,
    /// MAIL API Client
    mail_api_client: Option<MsftGraphApiEmail>,
    batch_size: usize,
    progress: Option<ProgressBar>,
}

impl MicrosoftImapResource {
    pub fn new(id: &str, secret: &str, mode: TokenGenerationMethod, config: &ImapConfig) -> Self {
        MicrosoftImapResource {
            client_id: id.to_string(),
            client_secret: secret.to_string(),
            redirect_uri: None,
            mode,
            auth_server: None,
            access_token: None,
            mail_api_client: None,
            batch_size: config.batch_size as usize,
            progress: if config.progress {
                Some(ProgressBar::new_spinner())
            } else {
                None
            },
        }
    }

    pub fn redirect_uri(&mut self, uri: Option<String>) -> &mut MicrosoftImapResource {
        self.redirect_uri = uri;
        self
    }

    pub fn server(
        &mut self,
        config: Option<Microsoft365AuthServerConfig>,
    ) -> &mut MicrosoftImapResource {
        self.auth_server = config;
        self
    }

    fn oauth_client(&self) -> OAuth {
        let mut oauth = OAuth::new();
        oauth
            .client_id(&self.client_id)
            .add_scope("files.read")
            .add_scope("Mail.Read")
            .add_scope("User.Read")
            .add_scope("offline_access")
            .access_token_url("https://login.microsoftonline.com/common/oauth2/v2.0/token")
            .refresh_token_url("https://login.microsoftonline.com/common/oauth2/v2.0/token");

        match self.mode {
            TokenGenerationMethod::AuthCode => {
                if self.auth_server.is_some() {
                    // can safely unwrap here
                    let server_config = self.auth_server.as_ref().unwrap();
                    oauth
                        .client_secret(&self.client_secret)
                        .authorize_url(
                            "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
                        )
                        .response_type("code")
                        .redirect_uri(&format!("{}{}", server_config.addr, server_config.base_url));
                } else {
                    warn!("The authenctication mode is auth code grant, but no server config was supplied");
                }
            }
            TokenGenerationMethod::DeviceCode => {
                oauth.authorize_url(
                    "https://login.microsoftonline.com/common/oauth2/v2.0/devicecode",
                );
            } // TokenGenerationMethod::ClientCredential => {
              //     let server_config = self.auth_server.as_ref().unwrap();
              //     println!("{server_config:#?}");
              //     oauth
              //         .add_scope("https://graph.microsoft.com/.default")
              //         .redirect_uri(&format!("{}{}", server_config.addr, server_config.base_url))
              //         .authorize_url("https://login.microsoftonline.com/common/adminconsent");
              // }
        };
        oauth
    }
}

#[async_trait]
impl ImapResource for MicrosoftImapResource {
    async fn init(&mut self) -> anyhow::Result<()> {
        let access_token = match self.mode {
            TokenGenerationMethod::AuthCode => {
                if let Some(auth_server) = &self.auth_server {
                    let (tx, mut rx) = mpsc::channel::<AccessToken>(32);
                    let client = self.oauth_client();
                    let auth_server = auth_server.clone();

                    tokio::spawn(async move {
                        auth_code::init_server(client, tx, auth_server.port).await;
                    });

                    rx.recv()
                        .await
                        .ok_or_else(|| anyhow!("Failed to receive access token"))?
                } else {
                    return Err(anyhow!("Server config absent"));
                }
            }
            // TokenGenerationMethod::ClientCredential => {
            //     let (tx, mut rx) = mpsc::channel::<AccessToken>(32);

            //     let config_clone = msft_365_config.clone();
            //     tokio::spawn(async move {
            //         client_credential::init_server(config_clone, tx).await;
            //     });

            //     rx.recv()
            //         .await
            //         .ok_or_else(|| anyhow!("Failed to receive access token"))?
            // }
            TokenGenerationMethod::DeviceCode => {
                device_code::init(Arc::new(Mutex::new(self.oauth_client())))
                    .await
                    .map_err(|err| anyhow!("{err}"))?
            }
        };

        self.mail_api_client = Some(MsftGraphApiEmail::new(&access_token));
        self.access_token = Some(access_token);

        Ok(())
    }

    fn username(&mut self) -> String {
        self.client_id.to_string()
    }

    async fn folders(&mut self) -> anyhow::Result<Vec<String>> {
        let client = self
            .mail_api_client
            .as_ref()
            .expect("Msft API client should be present");
        client.folders().await
    }

    async fn specified_folders(&mut self, folder_pattern: &str) -> anyhow::Result<Vec<Folder>> {
        let client = self
            .mail_api_client
            .as_ref()
            .expect("Msft API client should be present");
        client.specified_folders(folder_pattern).await
    }

    async fn process_messages_in_folder(&mut self, folder: &mut Folder) -> anyhow::Result<()> {
        let client = self
            .mail_api_client
            .as_ref()
            .expect("Email client should be present");
        let folder_name = folder.name.replace(' ', "");
        let mut all_messages = Vec::new();
        let mut skip_count = 0;

        loop {
            let batch_size = std::cmp::min(self.batch_size, 1000);
            let messages = client
                .messages(&folder_name, batch_size, skip_count)
                .await?;
            if messages.is_empty() {
                break;
            }
            all_messages.extend(messages);

            // Update skip_count for the next batch
            skip_count += batch_size;
        }
        folder.messages(all_messages);

        Ok(())
    }

    fn progress(&mut self) -> bool {
        self.progress.is_some()
    }
}

/// Credentials for Microsoft Graph API.
/// Enabling `surveilr` to get an `access_token` on behalf of the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Microsoft365Config {
    /// Client ID of the application from MSFT Azure App Directory
    pub client_id: String,
    /// Client Secret of the application from MSFT Azure App Directory
    pub client_secret: String,
    /// An optional redirect URL for `access_token` generation when using the AuthCode mode
    pub redirect_uri: Option<String>,
    /// The mode to generate an access_token. Default is 'DeviceCode'.
    pub mode: TokenGenerationMethod,
    /// Address to start the authentication server on. Used by the redirect_uri
    pub auth_server: Option<Microsoft365AuthServerConfig>,
}

// pub async fn retrieve_emails(
//     msft_365_config: &Microsoft365Config,
//     imap_config: &mut ImapConfig,
//     elaboration: &mut ImapElaboration,
// ) -> anyhow::Result<Vec<Folder>> {
//     emails::fetch_emails_from_graph_api(&access_token, imap_config, elaboration)
//         .await
//         .with_context(|| "[ingest_imap]: microsoft_365. Failed to fetch emails from graph api")
// }
