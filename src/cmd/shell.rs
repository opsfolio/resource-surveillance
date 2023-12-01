use super::ShellCommands;
use crate::resource::*;
use crate::shell::*;

// Implement methods for `AdminCommands`, ensure that whether the commands
// are called from CLI or natively within Rust, all the calls remain ergonomic.
impl ShellCommands {
    pub fn execute(&self, cli: &super::Cli, _args: &super::ShellArgs) -> anyhow::Result<()> {
        match self {
            ShellCommands::Json {
                command,
                cwd,
                stdout_only,
            } => self.json(cli, command, cwd.as_ref(), *stdout_only),
            ShellCommands::Plain { command, cwd } => self.plain(cli, command, cwd.as_ref()),
        }
    }

    fn json(
        &self,
        cli: &super::Cli,
        command: &str,
        _cwd: Option<&String>,
        _stdout_only: bool,
    ) -> anyhow::Result<()> {
        if cli.debug > 0 {
            println!("{:?}", command);
        }

        let stdin = crate::shell::ShellStdIn::None;
        let ce = CapturableExecutable::UriShellExecutive(
            Box::new(DenoTaskShellExecutive::new(command.to_owned(), None)),
            format!("cli://shell/result/{}", command),
            "json".to_owned(),
            false,
        );

        match ce.executed_result_as_json(stdin) {
            Ok((json_value, _nature, _is_sql_exec)) => {
                print!("{}", serde_json::to_string_pretty(&json_value).unwrap());
            }
            Err(err) => {
                print!("{:?}", err);
            }
        }

        Ok(())
    }

    fn plain(&self, cli: &super::Cli, command: &str, _cwd: Option<&String>) -> anyhow::Result<()> {
        if cli.debug > 0 {
            println!("{:?}", command);
        }

        let stdin = crate::shell::ShellStdIn::None;
        let ce = CapturableExecutable::UriShellExecutive(
            Box::new(DenoTaskShellExecutive::new(command.to_owned(), None)),
            format!("cli://shell/result/{}", command),
            "txt".to_owned(),
            false,
        );

        match ce.executed_result_as_text(stdin) {
            Ok((stdout, _nature, _is_sql_exec)) => {
                print!("{stdout}");
            }
            Err(err) => {
                print!("{:?}", err);
            }
        }

        Ok(())
    }
}
