use std::net::ToSocketAddrs;

use anyhow::{anyhow, Result};
use clap::Args;
use serde::Serialize;
use sqlpage::{
    app_config::{self, AppConfig},
    webserver, AppState,
};

use super::DEFAULT_STATEDB_FS_PATH;

/// SQLPage webserver
#[derive(Debug, Serialize, Args)]
pub struct SQLPageArgs {
    /// target SQLite database
    #[arg(short='d', long, default_value = DEFAULT_STATEDB_FS_PATH, default_missing_value = "always", env="SURVEILR_STATEDB_FS_PATH")]
    pub state_db_fs_path: String,

    #[arg(
        short = 'u',
        long,
        default_value = "/",
        default_missing_value = "always"
    )]
    pub url_base_path: String,

    /// Port to bind sqplage webserver to
    #[arg(short = 'p', long)]
    pub port: u16,
}

impl SQLPageArgs {
    pub fn execute(&self, args: &SQLPageArgs) -> Result<()> {
        // we're using just the single thread here since we're running just a webserver and multi threading will be excessive.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.block_on(self.start(args))?;
        Ok(())
    }

    fn database_url(&self, db_fs_path: &str) -> Result<String> {
        let prefix = "sqlite://".to_owned();
        let cwd = std::env::current_dir().unwrap_or_default();
        let db_path = cwd.join(db_fs_path);
        if let Ok(true) = db_path.try_exists() {
            return Ok(prefix + db_path.to_str().unwrap());
        } else {
            Err(anyhow!("Could not build database url for: {db_fs_path}"))
        }
    }

    // TODO use tracing crate for the logs
    async fn start(&self, args: &SQLPageArgs) -> Result<()> {
        let mut app_config = app_config::load()?;

        let addr = format!("0.0.0.0:{}", args.port).to_socket_addrs()?.next();
        app_config.listen_on = addr;
        app_config.database_url = self.database_url(&args.state_db_fs_path)?;

        // TODO change to debug!()
        println!("Starting with the following configuration: {app_config:#?}");

        let state = AppState::init(&app_config).await?;
        webserver::database::migrations::apply(&state.db).await?;

        // TODO change to info!()
        println!("Starting server...");
        self.log_welcome_message(&app_config);
        webserver::http::run_server(&app_config, state).await
    }

    fn log_welcome_message(&self, config: &AppConfig) {
        // Don't show 0.0.0.0 as the host, show the actual IP address
        let http_addr = config.listen_on().to_string().replace(
            "0.0.0.0",
            std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
                .to_string()
                .as_str(),
        );

        // TODO change to info!()
        println!(
            "Server started successfully.
    SQLPage is now running on {}
    You can add your website's code in .sql files to sqlpage_file table in {}.",
            if let Some(domain) = &config.https_domain {
                format!("https://{}", domain)
            } else {
                format!("http://{}", http_addr)
            },
            config.database_url
        );
    }
}
