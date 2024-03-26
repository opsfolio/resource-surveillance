use std::sync::Arc;

use anyhow::anyhow;
/// # Example
/// ```
/// use graph_rs_sdk::*:
///
/// #[tokio::main]
/// async fn main() {
///   start_server_main().await;
/// }
/// ```
///
/// # Overview:
///
/// [Microsoft Client Credentials](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-client-creds-grant-flow)
/// You can use the OAuth 2.0 client credentials grant specified in RFC 6749,
/// sometimes called two-legged OAuth, to access web-hosted resources by using the
/// identity of an application. This type of grant is commonly used for server-to-server
/// interactions that must run in the background, without immediate interaction with a user.
/// These types of applications are often referred to as daemons or service accounts.
///
/// This OAuth flow example requires signing in as an administrator for Azure, known as admin consent,
/// to approve your application to call Microsoft Graph Apis on behalf of a user. Admin consent
/// only has to be done once for a user. After admin consent is given, the oauth client can be
/// used to continue getting new access tokens programmatically.
use graph_rs_sdk::oauth::AccessToken;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::error;
use warp::Filter;

use crate::Microsoft365Config;

use super::oauth_client;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ClientCredentialsResponse {
    admin_consent: bool,
    tenant: String,
}

async fn request_access_token(config: &Microsoft365Config) -> anyhow::Result<AccessToken> {
    let mut oauth = oauth_client(config);
    let mut request = oauth.build_async().client_credentials();

    let response = request.access_token().send().await.unwrap();
    println!("{response:#?}");

    if response.status().is_success() {
        let access_token: AccessToken = response.json().await.unwrap();

        println!("{access_token:#?}");
        oauth.access_token(access_token.clone());
        Ok(access_token)
    } else {
        // See if Microsoft Graph returned an error in the Response body
        let result: reqwest::Result<serde_json::Value> = response.json().await;
        match result {
            Ok(body) => {
                if let Some(err) = body["error_description"].as_str() {
                    eprintln!("Failed to authenticate: {err}")
                };
                Err(anyhow!("{body:#?}"))
            }
            Err(err) => Err(anyhow!("Error on deserialization:\n{err:#?}")),
        }
    }
}

async fn handle_redirect(
    client_credential_option: Option<ClientCredentialsResponse>,
    config: &Microsoft365Config,
) -> anyhow::Result<AccessToken> {
    match client_credential_option {
        Some(client_credential_response) => {
            // Print out for debugging purposes.
            println!("{client_credential_response:#?}");

            // Request an access token.
            request_access_token(config).await
        }
        None => Err(anyhow!("Could not get client credential response")),
    }
}

pub async fn init_server(config: Microsoft365Config, tx: mpsc::Sender<AccessToken>) {
    let config = Arc::new(config);

    let query = warp::query::<ClientCredentialsResponse>()
        .map(Some)
        .or_else(|_| async {
            Ok::<(Option<ClientCredentialsResponse>,), std::convert::Infallible>((None,))
        });

    let config_for_routes = config.clone();

    let routes =
        warp::get()
            .and(warp::path("redirect"))
            .and(query)
            .and_then(move |client_credential| {
                let config_clone = config_for_routes.clone();
                let tx_clone = tx.clone();
                async move {
                    match handle_redirect(client_credential, &config_clone).await {
                        Ok(token) => {
                            if let Err(err) = tx_clone.send(token).await {
                                error!("Failed to send acccess token accross channel: {err:#?}");
                            };

                            Ok(Box::new(
                                "Successfully Logged In! You can close your browser.",
                            ))
                        }
                        Err(_) => Err(warp::reject()),
                    }
                }
            });

    // Get the oauth client and request a browser sign in
    let mut oauth = oauth_client(&config);
    let mut request = oauth.build_async().client_credentials();
    request
        .browser_authorization()
        .open()
        .expect("Failed to open server for authentication");
    let port = config.auth_server.clone().unwrap().port;

    println!(
        "Microsoft 365 Oauth Server Listening on: http://127.0.0.1:{}",
        port
    );

    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}
