use std::{collections::HashMap, fmt::Debug};

use async_trait::async_trait;
use derive_new::new;
use futures::{Sink, SinkExt};
use pgwire::{
    api::{
        auth::{
            finish_authentication, save_startup_parameters_to_metadata, AuthSource, LoginInfo,
            Password, ServerParameterProvider, StartupHandler,
        },
        ClientInfo, PgWireConnectionState,
    },
    error::{ErrorInfo, PgWireError, PgWireResult},
    messages::{
        response::ErrorResponse, startup::Authentication, PgWireBackendMessage,
        PgWireFrontendMessage,
    },
};
use tracing::{error, info};

use crate::auth::Auth;

#[async_trait]
impl AuthSource for Auth {
    async fn get_password(&self, login: &LoginInfo) -> PgWireResult<Password> {
        match login.user() {
            Some(user) => {
                if self.user() != user {
                    error!("No user matching this username was found. Got: {user}");
                    return Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                        "FATAL".to_string(),
                        "000".to_string(),
                        format!("No user matching this username was found. Got: {user}"),
                    ))));
                }
            }
            None => {
                error!("User not found");
                return Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                    "FATAL".to_string(),
                    "000".to_string(),
                    "No user found".to_string(),
                ))));
            }
        };

        let pass = self.password();
        Ok(Password::new(None, pass.as_bytes().to_vec()))
    }
}

#[derive(Debug, Clone, Default)]
pub struct UdiPgpParameters {
    version: String,
    date_style: String,
}

impl UdiPgpParameters {
    pub fn new() -> UdiPgpParameters {
        UdiPgpParameters {
            version: "15".into(),
            date_style: "ISO, MDY".into(),
        }
    }
}

impl ServerParameterProvider for UdiPgpParameters {
    fn server_parameters<C>(&self, _client: &C) -> Option<HashMap<String, String>>
    where
        C: ClientInfo,
    {
        let mut params = HashMap::with_capacity(4);
        params.insert("server_version".to_owned(), self.version.to_owned());
        params.insert("server_encoding".to_owned(), "UTF8".to_owned());
        params.insert("client_encoding".to_owned(), "UTF8".to_owned());
        params.insert("DateStyle".to_owned(), self.date_style.to_owned());
        params.insert("integer_datetimes".to_owned(), "on".to_owned());
        Some(params)
    }
}

#[derive(Debug, Clone, new)]
pub struct UdiPgpStartupHandler<A, P> {
    auth_source: A,
    parameter_provider: P,
}

#[async_trait]
impl<V: AuthSource, P: ServerParameterProvider> StartupHandler for UdiPgpStartupHandler<V, P> {
    async fn on_startup<C>(
        &self,
        client: &mut C,
        message: PgWireFrontendMessage,
    ) -> PgWireResult<()>
    where
        C: ClientInfo + Sink<PgWireBackendMessage> + Unpin + Send,
        C::Error: Debug,
        PgWireError: From<<C as Sink<PgWireBackendMessage>>::Error>,
    {
        info!("Initializing udi-pgp...");
        match message {
            PgWireFrontendMessage::Startup(ref startup) => {
                save_startup_parameters_to_metadata(client, startup);
                client.set_state(PgWireConnectionState::AuthenticationInProgress);
                client
                    .send(PgWireBackendMessage::Authentication(
                        Authentication::CleartextPassword,
                    ))
                    .await?;
            }
            PgWireFrontendMessage::PasswordMessageFamily(pwd) => {
                let pwd = pwd.into_password()?;
                let login_info = LoginInfo::from_client_info(client);
                let pass = self.auth_source.get_password(&login_info).await?;
                if pass.password() == pwd.password.as_bytes() {
                    finish_authentication(client, &self.parameter_provider).await
                } else {
                    let error_info = ErrorInfo::new(
                        "FATAL".to_owned(),
                        "28P01".to_owned(),
                        "Password authentication failed".to_owned(),
                    );
                    let error = ErrorResponse::from(error_info);

                    client
                        .feed(PgWireBackendMessage::ErrorResponse(error))
                        .await?;
                    client.close().await?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
