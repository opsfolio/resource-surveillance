use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::thread;

use deno_task_shell::execute_with_pipes;
use deno_task_shell::parser::parse;
use deno_task_shell::pipe;
use deno_task_shell::ShellPipeWriter;
use deno_task_shell::ShellState;
use serde_json::{json, Value};
use sha1::{Digest, Sha1};
use subprocess::ExitStatus;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

lazy_static::lazy_static! {
    pub static ref RUNTIME: Runtime = Runtime::new().expect("Failed to create Tokio runtime for Capturable Executables");
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ShellStdIn {
    None,
    Text(String),
    Json(serde_json::Value),
}

impl ShellStdIn {
    pub fn json(&self) -> Option<serde_json::Value> {
        match self {
            ShellStdIn::None => None,
            ShellStdIn::Text(text) => Some(serde_json::from_str(text.as_str()).unwrap()),
            ShellStdIn::Json(value) => Some(value.clone()),
        }
    }

    pub fn text(&self) -> Option<String> {
        match self {
            ShellStdIn::None => None,
            ShellStdIn::Text(text) => Some(text.clone()),
            ShellStdIn::Json(value) => Some(serde_json::to_string_pretty(&value).unwrap()),
        }
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.text().map(|s| s.into_bytes()).unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct ShellResult {
    pub status: ExitStatus,
    pub stderr: String,
    pub stdout: String,
}

#[allow(dead_code)]
impl ShellResult {
    pub fn success(&self) -> bool {
        matches!(self.status, ExitStatus::Exited(0))
    }

    pub fn json(&self) -> Value {
        let stdout_json = serde_json::from_str::<Value>(self.stdout.as_str());
        match stdout_json {
            Ok(json) => json!({
                "status": format!("{:?}", self.status),
                "stderr": self.stderr,
                "stdout": json
            }),
            Err(err) => json!({
                "status": format!("{:?}", self.status),
                "stderr": self.stderr,
                "stdout": self.stdout,
                "json-error": err.to_string()
            }),
        }
    }

    pub fn json_text(&self, default_json_text: Option<String>) -> String {
        let json = self.json();
        serde_json::to_string_pretty(&json).unwrap_or(if let Some(default) = default_json_text {
            default
        } else {
            r#"{ "error": "unable to serialize JSON with to_string_pretty" }"#.to_string()
        })
    }

    pub fn stdout_json_text(&self, default_json_text: Option<String>) -> String {
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

    pub fn stdout_hash(&self) -> String {
        let mut hasher = Sha1::new();
        hasher.update(self.stdout.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

pub fn execute_subprocess(
    command: impl AsRef<std::ffi::OsStr>,
    std_in: ShellStdIn,
) -> anyhow::Result<ShellResult> {
    let mut exec = subprocess::Exec::cmd(command)
        .stdout(subprocess::Redirection::Pipe)
        .stderr(subprocess::Redirection::Pipe);

    let stdin = std_in.text();
    if stdin.is_some() {
        exec = exec.stdin(subprocess::Redirection::Pipe);
    }

    let mut popen = exec.popen()?;

    if let Some(stdin_text) = stdin {
        if let Some(mut stdin_pipe) = popen.stdin.take() {
            stdin_pipe.write_all(stdin_text.as_bytes())?;
            stdin_pipe.flush()?;
            // `stdin_pipe` is dropped here when it goes out of scope, closing the stdin of the subprocess
        } // else: no one is listening to the stdin of the subprocess, so we can't pipe anything to it
    }

    let status = popen.wait()?;

    let mut output = String::new();
    popen.stdout.take().unwrap().read_to_string(&mut output)?;

    let mut error_output = String::new();
    match &mut popen.stderr.take() {
        Some(stderr) => {
            stderr.read_to_string(&mut error_output)?;
        }
        None => {}
    }

    Ok(ShellResult {
        status,
        stdout: output,
        stderr: error_output,
    })
}

pub trait ShellExecutive {
    fn execute(&self, stdin: ShellStdIn) -> anyhow::Result<ShellResult>;
}

impl ShellExecutive for String {
    fn execute(&self, stdin: ShellStdIn) -> anyhow::Result<ShellResult> {
        execute_subprocess(self, stdin)
    }
}

/// `ShellResultSupplier` provides a mechanism to execute shell commands and
/// capture their results using the `deno_task_shell` crate (cross-OS portable
/// shell).
///
/// It manages environment variables and can work with custom commands and
/// temporary directories. The primary method `result` executes a given shell
/// command and returns the exit code, stdout, and stderr outputs.
pub struct DenoTaskShellExecutive {
    // the parseable Deno Task Shell command text to execute
    pub command: String,
    // Environment variables to be used for commands.
    pub env_vars: HashMap<String, String>,
    // An optional working directory to execute commands in (defaults to env::current_dir).
    pub cwd: PathBuf,
    // An optional identity if we need to persist the output
    pub identity: Option<String>,
}

impl DenoTaskShellExecutive {
    /// Creates a new instance of `ShellResultSupplier` with default settings.
    ///
    /// It initializes environment variables from the current process environment.
    /// Custom commands and temporary directory are not set by default.
    pub fn new(command: String, identity: Option<String>) -> Self {
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

        Self {
            command,
            cwd: std::env::current_dir().unwrap_or(std::env::temp_dir()),
            env_vars,
            identity,
        }
    }

    pub fn _cwd(&mut self, path: &std::path::Path) -> &mut Self {
        self.cwd = path.to_path_buf();
        self
    }
}

impl ShellExecutive for DenoTaskShellExecutive {
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
    fn execute(&self, ce_stdin: ShellStdIn) -> anyhow::Result<ShellResult> {
        fn get_output_writer_and_handle() -> (ShellPipeWriter, JoinHandle<String>) {
            let (reader, writer) = pipe();
            let handle = reader.pipe_to_string_handle();
            (writer, handle)
        }

        // spawning a thread to handle the async part since making the "ShellExecutive" trait async at this moment faces some drawbacks.
        // 1. ShellResult is not Send and most of this "shell" module are not Send also, so using them in a full async context is not possible
        // until we can implement the Send trait for them all.
        // 2. One of the structs from the deno_task_shell crate has an Rc(Resource Counter) somewhere and it is by default
        // not Send which makes it impossible to implement Send

        let command = self.command.clone();
        let env_vars = self.env_vars.clone();
        let cwd = self.cwd.clone();

        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                match parse(&command) {
                    Ok(list) => {
                        let (stdin, mut stdin_writer) = pipe();
                        stdin_writer.write_all(&ce_stdin.bytes()).unwrap();
                        drop(stdin_writer); // prevent a deadlock by dropping the writer

                        let (stdout, stdout_handle) = get_output_writer_and_handle();
                        let (stderr, stderr_handle) = get_output_writer_and_handle();

                        let local_set = tokio::task::LocalSet::new();
                        let mut state = ShellState::new(env_vars.clone(), &cwd, Default::default());
                        state.apply_env_var("INIT_CWD", cwd.to_string_lossy().to_string().as_str());

                        let status = local_set
                            .run_until(execute_with_pipes(list, state, stdin, stdout, stderr))
                            .await;

                        let stderr = stderr_handle.await.unwrap();
                        let stdout = stdout_handle.await.unwrap();

                        Ok(ShellResult {
                            status: ExitStatus::Exited(status as u32),
                            stderr,
                            stdout,
                        })
                    }
                    Err(err) => Ok(ShellResult {
                        status: ExitStatus::Undetermined,
                        stderr: format!("{err:?}"),
                        stdout: String::new(),
                    }),
                }
            })
        });
        handle.join().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::shell::ShellExecutive;

    use super::DenoTaskShellExecutive;
    use super::ShellStdIn;

    #[test]
    fn test_command_execution() {
        let shell_result_supplier =
            DenoTaskShellExecutive::new(r#"echo "Hello, world!" | cat"#.to_string(), None);
        let result = shell_result_supplier.execute(ShellStdIn::None).unwrap();

        assert_eq!(result.status, subprocess::ExitStatus::Exited(0)); // Assuming 0 is the success code
        assert_eq!(result.stderr, ""); // Assuming no error message for a successful command
        assert_eq!(result.stdout.trim(), "Hello, world!");
    }

    #[test]
    fn test_environment_variable_handling() {
        // Set an environment variable and check if it's correctly passed
        let mut shell_result_supplier =
            DenoTaskShellExecutive::new("echo $TEST_VAR".to_string(), Some("test-ID".to_owned()));
        shell_result_supplier
            .env_vars
            .insert("TEST_VAR".to_string(), "123".to_string());

        let result = shell_result_supplier.execute(ShellStdIn::None).unwrap();
        assert_eq!(result.stdout.trim(), "123");
    }

    #[test]
    fn test_custom_command_handling() {
        // Implement this test based on how you're using custom commands
    }
}
