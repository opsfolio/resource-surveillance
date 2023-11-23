use std::path::Path;

use anyhow::Context;
use tokio::runtime::Runtime;

use super::ShellCommands;
use crate::shell::ShellResultSupplier;

// Implement methods for `AdminCommands`, ensure that whether the commands
// are called from CLI or natively within Rust, all the calls remain ergonomic.
impl ShellCommands {
    pub fn execute(&self, cli: &super::Cli, _args: &super::ShellArgs) -> anyhow::Result<()> {
        match self {
            ShellCommands::Json {
                command,
                cwd,
                stdout_only,
            } => self.result(cli, command, cwd.as_ref(), *stdout_only),
        }
    }

    fn result(
        &self,
        cli: &super::Cli,
        command: &str,
        cwd: Option<&String>,
        stdout_only: bool,
    ) -> anyhow::Result<()> {
        if cli.debug > 0 {
            println!("{:?}", command);
        }

        let runtime = Runtime::new()
            .with_context(|| "unable to create tokio Runtime in ShellCommands::result")?;
        let mut srs = ShellResultSupplier::new(cwd.map(|cwd| Path::new(cwd).to_path_buf()));
        let mut result = srs.result(&runtime, command, Default::default());

        print!(
            "{}",
            if stdout_only {
                result.stdout_json_text(None)
            } else {
                result.json_text(None)
            }
        );
        Ok(())
    }
}
