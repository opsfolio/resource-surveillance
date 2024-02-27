use std::fmt::Display;

use graph_rs_sdk::oauth::OAuth;
use serde::{Deserialize, Serialize};
use tracing::warn;

pub mod auth_code;
pub mod device_code;

/// The method for retrieving the access token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenGenerationMethod {
    /// Use the device code, that is, authenticate through another device.
    /// https://learn.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-device-code
    DeviceCode,
    /// Utilize a redirect url to get the access token
    AuthCode,
}

impl Display for TokenGenerationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenGenerationMethod::AuthCode => f.write_str("auth code grant"),
            TokenGenerationMethod::DeviceCode => f.write_str("device code"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsftAuthServerConfig {
    /// Address to bind the server to
    pub addr: String,
    /// Base redirect url. Defaults to `/redirect`
    pub base_url: String,
}

/// Credentials for Microsoft Graph API.
/// Enabling `surveilr` to get an `access_token` on behalf of the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsftConfig {
    /// Client ID of the application from MSFT Azure App Directory
    pub client_id: String,
    /// Client Secret of the application from MSFT Azure App Directory
    pub client_secret: String,
    /// An optional redirect URL for `access_token` generation when using the AuthCode mode
    pub redirect_uri: Option<String>,
    /// The mode to generate an access_token. Default is 'DeviceCode'.
    pub mode: TokenGenerationMethod,
    /// Address to start the authentication server on. Used by the redirect_uri
    pub auth_server: Option<MsftAuthServerConfig>,
}

fn oauth_client(creds: &MsftConfig) -> OAuth {
    let mut oauth = OAuth::new();
    oauth
        .client_id(&creds.client_id)
        .client_secret(&creds.client_secret)
        .add_scope("files.read")
        .add_scope("Mail.Read")
        .add_scope("User.Read")
        .add_scope("offline_access")
        .authorize_url("https://login.microsoftonline.com/common/oauth2/v2.0/authorize")
        .access_token_url("https://login.microsoftonline.com/common/oauth2/v2.0/token")
        .refresh_token_url("https://login.microsoftonline.com/common/oauth2/v2.0/token");

    match creds.mode {
        TokenGenerationMethod::AuthCode => {
            if creds.auth_server.is_some() {
                // can safely unwrap here
                let server_config = creds.auth_server.as_ref().unwrap();
                oauth
                    .response_type("code")
                    .redirect_uri(&format!("{}{}", server_config.addr, server_config.base_url));
            } else {
                warn!("The authenctication mode is auth code grant, but no server config was supplied");
            }
        }
        TokenGenerationMethod::DeviceCode => {}
    };
    oauth
}
