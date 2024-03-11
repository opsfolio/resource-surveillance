use graph_rs_sdk::oauth::AccessToken;
use graph_rs_sdk::{GraphFailure, GraphResult};
use std::process::exit;
use std::time::Duration;

use crate::msft::oauth_client;

use super::Microsoft365Config;

async fn poll_for_access_token(
    device_code: &str,
    interval: u64,
    message: &str,
    config: &Microsoft365Config,
) -> GraphResult<serde_json::Value> {
    let mut oauth = oauth_client(config);
    oauth.device_code(device_code);

    let mut request = oauth.build_async().device_code();
    let response = request.access_token().send().await?;

    let status = response.status();

    let body: serde_json::Value = response.json().await?;

    if !status.is_success() {
        loop {
            // Wait the amount of seconds that interval is.
            std::thread::sleep(Duration::from_secs(interval));

            let response = request.access_token().send().await?;

            let status = response.status();

            let body: serde_json::Value = response.json().await?;

            if status.is_success() {
                println!("Signed in successfully");
                return Ok(body);
            } else {
                let option_error = body["error"].as_str();

                if let Some(error) = option_error {
                    match error {
                        "authorization_pending" => println!("Still waiting on user to sign in"),
                        "authorization_declined" => panic!("user declined to sign in"),
                        "bad_verification_code" => println!("User is lost\n{message:#?}"),
                        "expired_token" => panic!("token has expired - user did not sign in"),
                        _ => {
                            eprintln!("This isn't the error we expected: {error:#?}");
                            exit(1);
                        }
                    }
                } else {
                    // Body should have error or we should bail.
                    panic!("Something went wrong. Please try to sign in again");
                }
            }
        }
    }

    Ok(body)
}

pub async fn init(config: &Microsoft365Config) -> GraphResult<AccessToken> {
    let mut oauth = oauth_client(config);

    let mut handler = oauth.build_async().device_code();
    let response = handler.authorization().send().await?;

    let json: serde_json::Value = response.json().await?;

    if let Some(err) = json["error_description"].as_str() {
        return Err(GraphFailure::Default {
            url: None,
            headers: None,
            message: err.to_string(),
        });
    }

    let device_code = json["device_code"].as_str().unwrap();
    let interval = json["interval"].as_u64().unwrap();
    let message = json["message"].as_str().unwrap();

    /*
    The authorization request is a POST and a successful response body will look similar to:

    Object {
        "device_code": String("FABABAAEAAAD--DLA3VO7QrddgJg7WevrgJ7Czy_TDsDClt2ELoEC8ePWFs"),
        "expires_in": Number(900),
        "interval": Number(5),
        "message": String("To sign in, use a web browser to open the page https://microsoft.com/devicelogin and enter the code FQK5HW3UF to authenticate."),
        "user_code": String("FQK5HW3UF"),
        "verification_uri": String("https://microsoft.com/devicelogin"),
    }
    */

    // Print the message to the user who needs to sign in:
    println!("{message:#?}");

    // Poll for the response to the token endpoint. This will go through once
    // the user has entered the code and signed in.
    let access_token_json = poll_for_access_token(device_code, interval, message, config).await?;
    let access_token: AccessToken = serde_json::from_value(access_token_json)?;

    // Get a refresh token. First pass the access token to the oauth instance.
    oauth.access_token(access_token.clone());
    let mut handler = oauth.build_async().device_code();

    let response = handler.refresh_token().send().await?;

    let _body: serde_json::Value = response.json().await?;

    Ok(access_token)
}