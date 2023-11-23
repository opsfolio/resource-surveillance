use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use deno_task_shell::execute_with_pipes;
use deno_task_shell::parser::parse;
use deno_task_shell::pipe;
use deno_task_shell::ShellCommand;
use deno_task_shell::ShellPipeWriter;
use deno_task_shell::ShellState;
use serde_json::{json, Value};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

pub struct ShellResult {
    pub status: i32,
    pub stderr: String,
    pub stdout: String,
}

impl ShellResult {
    pub fn json(&mut self) -> Value {
        let stdout_json = serde_json::from_str::<Value>(self.stdout.as_str());
        match stdout_json {
            Ok(json) => json!({
                "status": self.status,
                "stderr": self.stderr,
                "stdout": json
            }),
            Err(err) => json!({
                "status": self.status,
                "stderr": self.stderr,
                "stdout": self.stdout,
                "json-error": err.to_string()
            }),
        }
    }

    pub fn json_text(&mut self, default_json_text: Option<String>) -> String {
        let json = self.json();
        serde_json::to_string_pretty(&json).unwrap_or(if let Some(default) = default_json_text {
            default
        } else {
            r#"{ "error": "unable to serialize JSON with to_string_pretty" }"#.to_string()
        })
    }

    pub fn stdout_json_text(&mut self, default_json_text: Option<String>) -> String {
        let stdout_json = serde_json::from_str::<Value>(self.stdout.as_str());
        match stdout_json {
            Ok(json) => serde_json::to_string_pretty(&json).unwrap_or(
                if let Some(default) = default_json_text {
                    default
                } else {
                    r#"{ "error": "unable to serialize JSON with to_string_pretty" }"#.to_string()
                },
            ),
            Err(err) => {
                if let Some(default) = default_json_text {
                    default
                } else {
                    format!(r#"{{ "error": {:?} }}"#, err.to_string())
                }
            }
        }
    }
}

/// `ShellResultSupplier` provides a mechanism to execute shell commands and
/// capture their results using the `deno_task_shell` crate (cross-OS portable
/// shell).
///
/// It manages environment variables and can work with custom commands and
/// temporary directories. The primary method `result` executes a given shell
/// command and returns the exit code, stdout, and stderr outputs.
pub struct ShellResultSupplier {
    // Environment variables to be used for commands.
    env_vars: HashMap<String, String>,
    // Custom commands that can be used instead of the system shell commands.
    custom_commands: HashMap<String, Rc<dyn ShellCommand>>,
    // An optional working directory to execute commands in.
    cwd: PathBuf,
}

impl ShellResultSupplier {
    /// Creates a new instance of `ShellResultSupplier` with default settings.
    ///
    /// It initializes environment variables from the current process environment.
    /// Custom commands and temporary directory are not set by default.
    pub fn new(cwd: Option<PathBuf>) -> Self {
        let env_vars = std::env::vars()
            .map(|(key, value)| {
                // For some very strange reason, key will sometimes be cased as "Path"
                // or other times "PATH" on Windows. Since keys are case-insensitive on
                // Windows, normalize the keys to be upper case.
                if cfg!(windows) {
                    // need to normalize on windows
                    (key.to_uppercase(), value)
                } else {
                    (key, value)
                }
            })
            .collect();

        let cwd = if let Some(cwd) = cwd {
            cwd
        } else {
            std::env::temp_dir()
        };

        Self {
            cwd,
            env_vars,
            custom_commands: Default::default(),
        }
    }

    pub fn get_output_writer_and_handle(&self) -> (ShellPipeWriter, JoinHandle<String>) {
        let (reader, writer) = pipe();
        let handle = reader.pipe_to_string_handle();
        (writer, handle)
    }

    /// Executes a Deno task shell portable shell pipeline with the given stdin
    /// bytes and returns the results.
    ///
    /// The command is executed with the currently set environment variables and
    /// in the current working directory, or in a temporary directory if set.
    /// The function returns the exit code, stdout, and stderr as a tuple.
    ///
    /// # Arguments
    ///
    /// * `command` - A string slice that holds the command to be executed.
    /// * `stdin_bytes` - A vector of bytes that will be written to the command's
    ///    standard input.
    ///
    /// # Returns
    ///
    /// A tuple containing the exit code (`i32`), standard output (`String`),
    /// and standard error output (`String`).
    ///
    /// # Examples
    ///
    /// ```
    /// let mut supplier = ShellResultSupplier::new();
    /// let (exit_code, stdout, stderr) = supplier.result("echo Hello", Default::default());
    /// assert_eq!(stdout, "Hello\n");
    /// ```
    pub fn result(
        &mut self,
        runtime: &Runtime,
        command: &str,
        stdin_bytes: Vec<u8>,
    ) -> ShellResult {
        runtime.block_on(async {
            let list = parse(command).unwrap();
            let (stdin, mut stdin_writer) = pipe();
            stdin_writer.write_all(&stdin_bytes).unwrap();
            drop(stdin_writer); // prevent a deadlock by dropping the writer
            let (stdout, stdout_handle) = self.get_output_writer_and_handle();
            let (stderr, stderr_handle) = self.get_output_writer_and_handle();

            let local_set = tokio::task::LocalSet::new();
            let state = ShellState::new(
                self.env_vars.clone(),
                &self.cwd,
                self.custom_commands.drain().collect(),
            );
            let status = local_set
                .run_until(execute_with_pipes(list, state, stdin, stdout, stderr))
                .await;

            let stderr = stderr_handle.await.unwrap();
            let stdout = stdout_handle.await.unwrap();

            ShellResult {
                status,
                stderr,
                stdout,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use tokio::runtime::Runtime;

    use super::ShellResultSupplier;

    #[test]
    fn test_command_execution() {
        let mut shell_result_supplier = ShellResultSupplier::new(None);
        let runtime = Runtime::new().unwrap(); // Create a new Tokio runtime
        let command = r#"echo "Hello, world!" | cat"#;
        let result = shell_result_supplier.result(&runtime, command, vec![]);

        assert_eq!(result.status, 0); // Assuming 0 is the success code
        assert_eq!(result.stderr, ""); // Assuming no error message for a successful command
        assert_eq!(result.stdout.trim(), "Hello, world!");
    }

    #[test]
    fn test_environment_variable_handling() {
        // Set an environment variable and check if it's correctly passed
        let mut shell_result_supplier = ShellResultSupplier::new(None);
        shell_result_supplier
            .env_vars
            .insert("TEST_VAR".to_string(), "123".to_string());
        let command = "echo $TEST_VAR"; // Use appropriate syntax for your shell

        let runtime = Runtime::new().unwrap(); // Create a new Tokio runtime
        let result = shell_result_supplier.result(&runtime, command, vec![]);
        assert_eq!(result.stdout.trim(), "123");
    }

    #[test]
    fn test_custom_command_handling() {
        // Implement this test based on how you're using custom commands
    }
}
