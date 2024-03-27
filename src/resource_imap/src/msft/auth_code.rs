use std::sync::Arc;

use anyhow::anyhow;
use graph_rs_sdk::oauth::{AccessToken, OAuth};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error};
use warp::Filter;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AccessCode {
    code: String,
}

async fn set_and_req_access_code(
    access_code: AccessCode,
    oauth_client: Arc<Mutex<OAuth>>,
) -> anyhow::Result<AccessToken> {
    let mut oauth = oauth_client.lock().await;
    oauth.access_code(access_code.code.as_str());
    let mut request = oauth.build_async().authorization_code_grant();

    // Returns reqwest::Response
    let response = request.access_token().send().await?;
    debug!("SET_AND_REQ=={response:#?}");

    if response.status().is_success() {
        let mut access_token: AccessToken = response.json().await?;
        let jwt = access_token.jwt();
        debug!("JWT==={jwt:#?}");

        oauth.access_token(access_token.clone());
        debug!("ACCESS_TOKEN{:#?}", &oauth);

        drop(oauth);

        Ok(access_token)
    } else {
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
    code_option: Option<AccessCode>,
    oauth_client: Arc<Mutex<OAuth>>,
) -> anyhow::Result<AccessToken> {
    match code_option {
        Some(access_code) => {
            // Print out the code for debugging purposes.
            debug!("ACCESS_CODE=={access_code:#?}");
            set_and_req_access_code(access_code, oauth_client).await
        }
        None => Err(anyhow!("Could not get access code")),
    }
}

pub async fn init_server(oauth_client: OAuth, tx: mpsc::Sender<AccessToken>, port: u16) {
    let client = Arc::new(Mutex::new(oauth_client));

    let query = warp::query::<AccessCode>()
        .map(Some)
        .or_else(|_| async { Ok::<(Option<AccessCode>,), std::convert::Infallible>((None,)) });

    let client_for_routes = client.clone();

    let routes = warp::get()
        .and(warp::path("redirect"))
        .and(query)
        .and_then(move |code| {
            let client_clone = client.clone();
            let tx_clone = tx.clone();

            async move {
                match handle_redirect(code, client_clone).await {
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

    let mut client = client_for_routes.lock().await;
    client
        .build_async()
        .authorization_code_grant()
        .browser_authorization()
        .open()
        .expect("Failed to open browser for OAuth");

    println!(
        "Microsoft 365 Oauth Server Listening on: http://127.0.0.1:{}",
        port
    );
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}
