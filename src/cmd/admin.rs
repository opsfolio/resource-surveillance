use super::AdminCommands;

// Implement methods for `NotebooksCommands`, ensure that whether the commands
// are called from CLI or natively within Rust, all the calls remain ergonomic.
impl AdminCommands {
    pub fn execute(&self, _cli: &super::Cli, args: &super::AdminArgs) -> anyhow::Result<()> {
        match self {
            AdminCommands::CliHelpMd => self.ls(args),
        }
    }

    fn ls(&self, _args: &super::AdminArgs) -> anyhow::Result<()> {
        clap_markdown::print_help_markdown::<super::Cli>();
        Ok(())
    }
}
