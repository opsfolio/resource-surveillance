use std::sync::Arc;

use anyhow::anyhow;
use graph_rs_sdk::oauth::AccessToken;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use warp::Filter;

use crate::msft::oauth_client;
use crate::Microsoft365Config;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AccessCode {
    code: String,
}

async fn set_and_req_access_code(
    access_code: AccessCode,
    config: &Microsoft365Config,
) -> anyhow::Result<AccessToken> {
    let mut oauth = oauth_client(config);
    // The response type is automatically set to token and the grant type is automatically
    // set to authorization_code if either of these were not previously set.
    // This is done here as an example.
    oauth.access_code(access_code.code.as_str());
    let mut request = oauth.build_async().authorization_code_grant();

    // Returns reqwest::Response
    let response = request.access_token().send().await?;
    println!("{response:#?}");

    if response.status().is_success() {
        let mut access_token: AccessToken = response.json().await?;

        // Option<&JsonWebToken>
        let jwt = access_token.jwt();
        println!("{jwt:#?}");

        // Store in OAuth to make requests for refresh tokens.
        oauth.access_token(access_token.clone());

        // If all went well here we can print out the OAuth config with the Access Token.
        println!("{:#?}", &oauth);
        Ok(access_token)
    } else {
        // See if Microsoft Graph returned an error in the Response body
        let result: reqwest::Result<serde_json::Value> = response.json().await;

        match result {
            Ok(body) => Err(anyhow!("{body:#?}")),
            Err(err) => Err(anyhow!("Error on deserialization:\n{err:#?}")),
        }
    }
}

async fn handle_redirect(
    code_option: Option<AccessCode>,
    config: &Microsoft365Config,
) -> anyhow::Result<AccessToken> {
    match code_option {
        Some(access_code) => {
            // Print out the code for debugging purposes.
            println!("{access_code:#?}");

            // Set the access code and request an access token.
            // Callers should handle the Result from requesting an access token
            // in case of an error here.
            set_and_req_access_code(access_code, config).await
        }
        None => Err(anyhow!("Could not get access code")),
    }
}

pub async fn init_server(config: Microsoft365Config, tx: mpsc::Sender<AccessToken>) {
    let config = Arc::new(config);

    let query = warp::query::<AccessCode>()
        .map(Some)
        .or_else(|_| async { Ok::<(Option<AccessCode>,), std::convert::Infallible>((None,)) });

    let config_for_routes = config.clone();

    let routes = warp::get()
        .and(warp::path("redirect"))
        .and(query)
        .and_then(move |code| {
            let config_clone = config_for_routes.clone();
            let tx_clone = tx.clone(); // Clone the sender for use in the closure

            async move {
                match handle_redirect(code, &config_clone).await {
                    Ok(token) => {
                        if let Err(err) = tx_clone.send(token).await {
                            eprintln!("Failed to send acccess token accross channel: {err:#?}");
                        };

                        Ok(Box::new(
                            "Successfully Logged In! You can close your browser.",
                        ))
                    }
                    Err(_) => Err(warp::reject()),
                }
            }
        });

    let config_for_oauth = config.clone();

    oauth_client(&config_for_oauth)
        .build_async()
        .authorization_code_grant()
        .browser_authorization()
        .open()
        .expect("Failed to open browser for OAuth");

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
