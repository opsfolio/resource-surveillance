use sha1::{Digest, Sha1};
use std::error::Error;
use std::io::{Read, Write};

use crate::resource::*;
use tokio::runtime::Runtime;

lazy_static::lazy_static! {
    pub static ref RUNTIME: Runtime = Runtime::new().expect("Failed to create Tokio runtime for Capturable Executables");
}

pub type BinaryExecOutput = (
    Box<dyn BinaryContent>,
    subprocess::ExitStatus,
    Option<String>,
);
pub type BinaryExecOutputSupplier =
    Box<dyn Fn(Option<String>) -> Result<BinaryExecOutput, Box<dyn std::error::Error>>>;

pub type TextExecOutput = (Box<dyn TextContent>, subprocess::ExitStatus, Option<String>);
pub type TextExecOutputSupplier =
    Box<dyn Fn(Option<String>) -> Result<TextExecOutput, Box<dyn std::error::Error>>>;

#[derive(Debug, Clone)]
pub enum CapturableExecutableStdIn {
    None,
    Text(String),
    Json(serde_json::Value),
}

impl CapturableExecutableStdIn {
    pub fn from_json(value: serde_json::Value) -> CapturableExecutableStdIn {
        CapturableExecutableStdIn::Json(value)
    }

    pub fn text(&self) -> Option<String> {
        match self {
            CapturableExecutableStdIn::None => None,
            CapturableExecutableStdIn::Text(text) => Some(text.clone()),
            CapturableExecutableStdIn::Json(value) => {
                Some(serde_json::to_string_pretty(&value).unwrap())
            }
        }
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.text().map(|s| s.into_bytes()).unwrap_or_default()
    }
}

pub fn execution_result_text(
    uri: &str,
    std_in: CapturableExecutableStdIn,
) -> Result<TextExecOutput, Box<dyn Error>> {
    let mut exec = subprocess::Exec::cmd(uri)
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

    let hash = {
        let mut hasher = Sha1::new();
        hasher.update(output.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    Ok((
        Box::new(ResourceTextContent { hash, text: output }) as Box<dyn TextContent>,
        status,
        if !error_output.is_empty() {
            Some(error_output)
        } else {
            None
        },
    ))
}

pub fn execution_result_binary(
    uri: &str,
    std_in: CapturableExecutableStdIn,
) -> Result<BinaryExecOutput, Box<dyn Error>> {
    let mut exec = subprocess::Exec::cmd(uri)
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

    let mut output = popen.stdout.take().unwrap();
    let mut binary = Vec::new();
    output.read_to_end(&mut binary)?;

    let mut error_output = String::new();
    match &mut popen.stderr.take() {
        Some(stderr) => {
            stderr.read_to_string(&mut error_output)?;
        }
        None => {}
    }

    let hash = {
        let mut hasher = Sha1::new();
        hasher.update(&binary);
        format!("{:x}", hasher.finalize())
    };

    Ok((
        Box::new(ResourceBinaryContent { hash, binary }) as Box<dyn BinaryContent>,
        status,
        if !error_output.is_empty() {
            Some(error_output)
        } else {
            None
        },
    ))
}

/// Return a TextExecOutputSupplier for URI so that it can be used as a closure
pub fn executable_content_text(uri: &str) -> TextExecOutputSupplier {
    let uri = uri.to_string(); // Clone for closure's lifetime
    Box::new(move |stdin| -> Result<TextExecOutput, Box<dyn Error>> {
        execution_result_text(
            &uri,
            if let Some(stdin) = stdin {
                CapturableExecutableStdIn::Text(stdin)
            } else {
                CapturableExecutableStdIn::None
            },
        )
    })
}

/// Return a BinaryExecOutputSupplier for URI so that it can be used as a closure
pub fn executable_content_binary(uri: &str) -> BinaryExecOutputSupplier {
    let uri = uri.to_string(); // Clone for closure's lifetime
    Box::new(
        move |stdin| -> Result<BinaryExecOutput, Box<dyn std::error::Error>> {
            execution_result_binary(
                &uri,
                if let Some(stdin) = stdin {
                    CapturableExecutableStdIn::Text(stdin)
                } else {
                    CapturableExecutableStdIn::None
                },
            )
        },
    )
}
