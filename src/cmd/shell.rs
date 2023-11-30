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
            } => self.result(cli, command, cwd.as_ref(), *stdout_only),
        }
    }

    fn result(
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
            Box::new(DenoTaskShellExecutive::new(command.to_string())),
            format!("cli://shell/result/{}", command),
            String::from("json"),
            false,
        );

        let (json_value, _nature, _) = ce.executed_result_as_json(stdin).unwrap();
        print!("{}", serde_json::to_string_pretty(&json_value).unwrap());

        Ok(())
    }
}
