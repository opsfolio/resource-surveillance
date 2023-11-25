use sha1::{Digest, Sha1};
use std::error::Error;
use std::io::{Read, Write};

use is_executable::IsExecutable;
use regex::{Regex, RegexSet};

use crate::fscontent::*;
use crate::resource::*;

#[derive(Debug, Clone)]
pub enum CapturableExecutable {
    Text(String, bool),
    RequestedButNoNature(Regex),
    RequestedButNotExecutable,
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

impl CapturableExecutable {
    pub fn executable_content(
        &self,
        uri: &str,
    ) -> (
        Option<TextExecOutputSupplier>,
        Option<BinaryExecOutputSupplier>,
    ) {
        let capturable_exec_text_supplier: Option<TextExecOutputSupplier>;
        let capturable_exec_binary_supplier: Option<BinaryExecOutputSupplier>;

        if !matches!(self, CapturableExecutable::RequestedButNotExecutable) {
            let uri_clone_cebs = uri.to_string(); // Clone for the first closure
            capturable_exec_binary_supplier = Some(Box::new(
                move |stdin| -> Result<BinaryExecOutput, Box<dyn std::error::Error>> {
                    let mut exec = subprocess::Exec::cmd(&uri_clone_cebs)
                        .stdout(subprocess::Redirection::Pipe);

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
                        Box::new(FileBinaryContent { hash, binary }) as Box<dyn BinaryContent>,
                        status,
                        if !error_output.is_empty() {
                            Some(error_output)
                        } else {
                            None
                        },
                    ))
                },
            ));

            let uri_clone_cets = uri.to_string(); // Clone for the second closure
            capturable_exec_text_supplier = Some(Box::new(
                move |stdin| -> Result<TextExecOutput, Box<dyn Error>> {
                    let mut exec = subprocess::Exec::cmd(&uri_clone_cets)
                        .stdout(subprocess::Redirection::Pipe)
                        .stderr(subprocess::Redirection::Pipe);

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
                        Box::new(FileTextContent { hash, text: output }) as Box<dyn TextContent>,
                        status,
                        if !error_output.is_empty() {
                            Some(error_output)
                        } else {
                            None
                        },
                    ))
                },
            ));
        } else {
            capturable_exec_binary_supplier = None;
            capturable_exec_text_supplier = None;
        }

        (
            capturable_exec_text_supplier,
            capturable_exec_binary_supplier,
        )
    }
}

const DEFAULT_CAPTURE_EXEC_REGEX_PATTERN: &str = r"surveilr\[(?P<nature>[^\]]*)\]";
const DEFAULT_CAPTURE_SQL_EXEC_REGEX_PATTERN: &str = r"surveilr-SQL";

pub trait CapturableExecutableSupplier {
    fn capturable_executable(&self) -> Option<CapturableExecutable>;
}

pub struct CapturableExecutableRegexRules {
    pub capturable_regexs: Vec<Regex>,
    pub capturable_sql_set: RegexSet,
}

impl CapturableExecutableRegexRules {
    pub fn new(
        capturable_executables_regexs: Option<&[regex::Regex]>,
        captured_exec_sql_regexs: Option<&[regex::Regex]>,
    ) -> anyhow::Result<Self> {
        // Constructor can fail due to RegexSet::new
        let is_capturable = match capturable_executables_regexs {
            Some(capturable_executables_regexs) => capturable_executables_regexs.to_vec(),
            None => vec![Regex::new(DEFAULT_CAPTURE_EXEC_REGEX_PATTERN)?],
        };
        let is_capturable_sql = match captured_exec_sql_regexs {
            Some(captured_exec_sql_regexs) => {
                RegexSet::new(captured_exec_sql_regexs.iter().map(|r| r.as_str()))?
            }
            None => RegexSet::new([DEFAULT_CAPTURE_SQL_EXEC_REGEX_PATTERN])?,
        };

        Ok(CapturableExecutableRegexRules {
            capturable_regexs: is_capturable,
            capturable_sql_set: is_capturable_sql,
        })
    }

    pub fn capturable_executable(&self, path: &std::path::Path) -> Option<CapturableExecutable> {
        let mut ce: Option<CapturableExecutable> = None;
        let haystack: &str = path.to_str().unwrap();

        if self.capturable_sql_set.is_match(haystack) {
            ce = Some(CapturableExecutable::Text(
                String::from("surveilr-SQL"),
                true,
            ));
        } else {
            for re in self.capturable_regexs.iter() {
                if let Some(caps) = re.captures(haystack) {
                    if let Some(nature) = caps.name("nature") {
                        ce = Some(CapturableExecutable::Text(
                            String::from(nature.as_str()),
                            false,
                        ));
                        break;
                    } else {
                        ce = Some(CapturableExecutable::RequestedButNoNature(re.clone()));
                        break;
                    }
                }
            }
        }
        if ce.is_some() {
            if path.is_executable() {
                return ce;
            } else {
                return Some(CapturableExecutable::RequestedButNotExecutable);
            }
        }
        None
    }
}
