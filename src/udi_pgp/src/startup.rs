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
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info};

use crate::{
    config::{manager::Message, UdiPgpConfig},
    error::{UdiPgpError, UdiPgpResult},
    processor::UdiPgpProcessor,
};

pub struct UdiPgpAuthSource {
    config_tx: mpsc::Sender<Message>,
}

impl UdiPgpAuthSource {
    pub fn new(tx: mpsc::Sender<Message>) -> Self {
        Self { config_tx: tx }
    }

    async fn read_config(&self) -> UdiPgpResult<UdiPgpConfig> {
        let (response_tx, response_rx) = oneshot::channel();
        let read_state_msg = Message::ReadConfig(response_tx);
        self.config_tx
            .send(read_state_msg)
            .await
            .expect("Failed to send message");
        match response_rx.await {
            Ok(state) => {
                debug!("Latest Config: {:#?}", state);
                Ok(state)
            }
            Err(e) => {
                error!("{}", e);
                Err(UdiPgpError::ConfigError(format!(
                    "Failed to read configuration: {}",
                    e
                )))
            }
        }
    }
}

#[async_trait]
impl AuthSource for UdiPgpAuthSource {
    async fn get_password(&self, login: &LoginInfo) -> PgWireResult<Password> {
        let (supplier_id, _) = UdiPgpProcessor::extract_supplier_and_database(login.database())?;

        let user = match login.user() {
            Some(user) => user,
            None => {
                let err_msg = "Login information does not include a user.";
                error!("{}", err_msg);
                return Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                    "FATAL".to_string(),
                    "AUTH".to_string(),
                    err_msg.to_string(),
                ))));
            }
        };

        let config = self.read_config().await?;
        let auth = match config.supplier_auth(&supplier_id, user)? {
            Some(auth) => auth,
            None => {
                let err_msg = format!("No user matching this username was found. Got: {}", user);
                error!("{}", err_msg);
                return Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                    "FATAL".to_string(),
                    "003".to_string(),
                    err_msg,
                ))));
            }
        };

        Ok(Password::new(None, auth.password().as_bytes().to_vec()))
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
    config_tx: mpsc::Sender<Message>,
}

impl<V: AuthSource, P: ServerParameterProvider> UdiPgpStartupHandler<V, P> {
    async fn read_config(&self) -> UdiPgpResult<UdiPgpConfig> {
        let (response_tx, response_rx) = oneshot::channel();
        let read_state_msg = Message::ReadConfig(response_tx);
        self.config_tx
            .send(read_state_msg)
            .await
            .expect("Failed to send message");
        match response_rx.await {
            Ok(state) => {
                debug!("Latest Config: {:#?}", state);
                Ok(state)
            }
            Err(e) => {
                error!("{}", e);
                Err(UdiPgpError::ConfigError(format!(
                    "Failed to read configuration: {}",
                    e
                )))
            }
        }
    }
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

        // that is, no supplier, just the admin supplier
        match message {
            PgWireFrontendMessage::Startup(ref startup) => {
                save_startup_parameters_to_metadata(client, startup);

                let config = self.read_config().await?;
                if config.suppliers.is_empty() {
                    finish_authentication(client, &self.parameter_provider).await;
                    return Ok(());
                }

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
