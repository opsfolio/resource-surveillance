use udi_pgp::cli::UdiPgpArgs;


pub async fn execute(args: &UdiPgpArgs) -> anyhow::Result<()> {
    let config = udi_pgp::config::UdiPgpConfig::from(args);
    udi_pgp::run(&config).await
}
