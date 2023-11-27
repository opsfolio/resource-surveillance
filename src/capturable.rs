use is_executable::IsExecutable;
use regex::{Regex, RegexSet};

use crate::shell::*;
use crate::subprocess::*;

#[derive(Debug, Clone)]
pub enum CapturableExecutable {
    TextFromExecutableUri(String, String, bool),
    TextFromDenoTaskShellCmd(String, String, String, bool),
    RequestedButNoNature(String, Regex),
    RequestedButNotExecutable(String),
}

impl CapturableExecutable {
    pub fn executed_result_as_text(
        &self,
        std_in: CapturableExecutableStdIn,
    ) -> anyhow::Result<(String, String, bool), serde_json::Value> {
        match self {
            CapturableExecutable::TextFromExecutableUri(src, nature, is_batched_sql) => {
                match execution_result_text(src, std_in) {
                    Ok((stdout, status, stderr)) => {
                        if status.success() {
                            Ok((
                                String::from(stdout.content_text()),
                                nature.clone(),
                                *is_batched_sql,
                            ))
                        } else {
                            Err(serde_json::json!({
                                "src": src,
                                "issue": "[CapturableExecutable::TextFromExecutableUri.executed_text] invalid exit status",
                                "remediation": "ensure that executable is called with proper arguments and input formats",
                                "nature": nature,
                                "exit-status": format!("{:?}", status),
                                "stdout": stdout.content_text(),
                                "stderr": stderr
                            }))
                        }
                    }
                    Err(err) => Err(serde_json::json!({
                        "src": src,
                        "issue": "[CapturableExecutable::TextFromExecutableUri.executed_text] execution error",
                        "rust-err": format!("{:?}", err),
                        "nature": nature,
                    })),
                }
            }
            CapturableExecutable::TextFromDenoTaskShellCmd(uri, src, nature, is_batched_sql) => {
                let mut srs = ShellResultSupplier::new(None);
                let shell_result = srs.result(&RUNTIME, src, std_in.bytes());
                if shell_result.status == 0 {
                    Ok((shell_result.stdout, nature.clone(), *is_batched_sql))
                } else {
                    Err(serde_json::json!({
                        "uri": uri,
                        "src": src,
                        "issue": "[CapturableExecutable::TextFromDenoTaskShell.executed_text] invalid exit status",
                        "remediation": "ensure that executable is called with proper arguments and input formats",
                        "nature": nature,
                        "exit-status": format!("{:?}", shell_result.status),
                        "stdout": shell_result.stdout,
                        "stderr": shell_result.stderr
                    }))
                }
            }
            CapturableExecutable::RequestedButNoNature(src, regex) => Err(serde_json::json!({
                "src": src,
                "issue": "[CapturableExecutable::RequestedButNoNature.executed_sql] unable to determine nature",
                "remediation": "make sure that the regular expression has a `nature` named capture group",
                "regex": format!("{:?}", regex),
            })),
            CapturableExecutable::RequestedButNotExecutable(src) => Err(serde_json::json!({
                "src": src,
                "issue": "[CapturableExecutable::RequestedButNotExecutable.executed_sql] executable permissions not set",
                "remediation": "make sure that script has executable permissions set",
            })),
        }
    }

    pub fn executed_result_as_json(
        &self,
        std_in: CapturableExecutableStdIn,
    ) -> anyhow::Result<(serde_json::Value, String, bool), serde_json::Value> {
        match self {
            CapturableExecutable::TextFromExecutableUri(src, nature, is_batched_sql) => {
                match execution_result_text(src, std_in) {
                    Ok((stdout, status, stderr)) => {
                        if status.success() {
                            let captured_text = String::from(stdout.content_text());
                            let value: serde_json::Result<serde_json::Value> =
                                serde_json::from_str(&captured_text);
                            match value {
                                Ok(value) => Ok((value, nature.clone(), *is_batched_sql)),
                                Err(_) => Err(serde_json::json!({
                                    "src": src,
                                    "issue": "[CapturableExecutable::TextFromExecutableUri.executed_result_as_json] unable to deserialize JSON",
                                    "remediation": "ensure that executable is emitting JSON (e.g. `--json`)",
                                    "nature": nature,
                                    "is-batched-sql": is_batched_sql,
                                    "stdout": captured_text,
                                    "exit-status": format!("{:?}", status),
                                    "stderr": stderr
                                })),
                            }
                        } else {
                            Err(serde_json::json!({
                                "src": src,
                                "issue": "[CapturableExecutable::TextFromExecutableUri.executed_result_as_json] invalid exit status",
                                "remediation": "ensure that executable is called with proper arguments and input formats",
                                "nature": nature,
                                "is-batched-sql": is_batched_sql,
                                "exit-status": format!("{:?}", status),
                                "stderr": stderr
                            }))
                        }
                    }
                    Err(err) => Err(serde_json::json!({
                        "src": src,
                        "issue": "[CapturableExecutable::TextFromExecutableUri.executed_result_as_json] execution error",
                        "rust-err": format!("{:?}", err),
                        "nature": nature,
                        "is-batched-sql": is_batched_sql,
                    })),
                }
            }
            CapturableExecutable::TextFromDenoTaskShellCmd(uri, src, nature, is_batched_sql) => {
                let mut srs = ShellResultSupplier::new(None);
                let shell_result = srs.result(&RUNTIME, src, std_in.bytes());
                if shell_result.status == 0 {
                    let value: serde_json::Result<serde_json::Value> =
                        serde_json::from_str(&shell_result.stdout);
                    match value {
                        Ok(value) => Ok((value, nature.clone(), *is_batched_sql)),
                        Err(_) => Err(serde_json::json!({
                            "uri": uri,
                            "src": src,
                            "issue": "[CapturableExecutable::TextFromDenoTaskShell.executed_result_as_json] unable to deserialize JSON",
                            "remediation": "ensure that executable is emitting JSON (e.g. `--json`)",
                            "nature": nature,
                            "is-batched-sql": is_batched_sql,
                            "stdout": shell_result.stdout,
                            "exit-status": format!("{:?}", shell_result.status),
                            "stderr": shell_result.stderr
                        })),
                    }
                } else {
                    Err(serde_json::json!({
                        "uri": uri,
                        "src": src,
                        "issue": "[CapturableExecutable::TextFromDenoTaskShell.executed_result_as_json] invalid exit status",
                        "remediation": "ensure that executable is called with proper arguments and input formats",
                        "nature": nature,
                        "exit-status": format!("{:?}", shell_result.status),
                        "stdout": shell_result.stdout,
                        "stderr": shell_result.stderr
                    }))
                }
            }
            CapturableExecutable::RequestedButNoNature(src, regex) => Err(serde_json::json!({
                "src": src,
                "issue": "[CapturableExecutable::RequestedButNoNature.executed_result_as_json] unable to determine nature",
                "remediation": "make sure that the regular expression has a `nature` named capture group",
                "regex": format!("{:?}", regex),
            })),
            CapturableExecutable::RequestedButNotExecutable(src) => Err(serde_json::json!({
                "src": src,
                "issue": "[CapturableExecutable::RequestedButNotExecutable.executed_result_as_json] executable permissions not set",
                "remediation": "make sure that script has executable permissions set",
            })),
        }
    }

    pub fn executed_result_as_sql(
        &self,
        std_in: CapturableExecutableStdIn,
    ) -> anyhow::Result<(String, String), serde_json::Value> {
        match self {
            CapturableExecutable::TextFromExecutableUri(src, nature, is_batched_sql) => {
                if *is_batched_sql {
                    match execution_result_text(src, std_in) {
                        Ok((stdout, status, stderr)) => {
                            if status.success() {
                                Ok((String::from(stdout.content_text()), nature.clone()))
                            } else {
                                Err(serde_json::json!({
                                    "src": src,
                                    "issue": "[CapturableExecutable::TextFromExecutableUri.executed_result_as_sql] invalid exit status",
                                    "remediation": "ensure that executable is called with proper arguments and input formats",
                                    "nature": nature,
                                    "exit-status": format!("{:?}", status),
                                    "stdout": stdout.content_text(),
                                    "stderr": stderr
                                }))
                            }
                        }
                        Err(err) => Err(serde_json::json!({
                            "src": src,
                            "issue": "[CapturableExecutable::TextFromExecutableUri.executed_result_as_sql] execution error",
                            "rust-err": format!("{:?}", err),
                            "nature": nature,
                        })),
                    }
                } else {
                    Err(serde_json::json!({
                        "src": src,
                        "issue": "[CapturableExecutable::TextFromExecutableUri.executed_result_as_sql] is not classified as batch SQL",
                        "nature": nature,
                    }))
                }
            }
            CapturableExecutable::TextFromDenoTaskShellCmd(uri, src, nature, is_batched_sql) => {
                let mut srs = ShellResultSupplier::new(None);
                let shell_result = srs.result(&RUNTIME, src, std_in.bytes());
                if *is_batched_sql {
                    if shell_result.status == 0 {
                        Ok((shell_result.stdout, nature.clone()))
                    } else {
                        Err(serde_json::json!({
                            "uri": uri,
                            "src": src,
                            "issue": "[CapturableExecutable::TextFromDenoTaskShell.executed_result_as_sql] invalid exit status",
                            "remediation": "ensure that executable is called with proper arguments and input formats",
                            "nature": nature,
                            "exit-status": format!("{:?}", shell_result.status),
                            "stdout": shell_result.stdout,
                            "stderr": shell_result.stderr
                        }))
                    }
                } else {
                    Err(serde_json::json!({
                        "uri": uri,
                        "src": src,
                        "issue": "[CapturableExecutable::TextFromDenoTaskShell.executed_result_as_sql] is not classified as batch SQL",
                        "nature": nature,
                    }))
                }
            }
            CapturableExecutable::RequestedButNoNature(src, regex) => Err(serde_json::json!({
                "src": src,
                "issue": "[CapturableExecutable::RequestedButNoNature.executed_result_as_sql] unable to determine nature",
                "remediation": "make sure that the regular expression has a `nature` named capture group",
                "regex": format!("{:?}", regex),
            })),
            CapturableExecutable::RequestedButNotExecutable(src) => Err(serde_json::json!({
                "src": src,
                "issue": "[CapturableExecutable::RequestedButNotExecutable.executed_result_as_sql] executable permissions not set",
                "remediation": "make sure that script has executable permissions set",
            })),
        }
    }

    pub fn executable_content_text(&self) -> Option<TextExecOutputSupplier> {
        match self {
            CapturableExecutable::TextFromExecutableUri(uri, _, _) => {
                Some(executable_content_text(uri))
            }
            CapturableExecutable::TextFromDenoTaskShellCmd(_uri, src, _, _) => {
                Some(deno_task_shell_content_text(src))
            }
            CapturableExecutable::RequestedButNoNature(uri, _) => {
                Some(executable_content_text(uri))
            }
            CapturableExecutable::RequestedButNotExecutable(_) => None,
        }
    }

    pub fn executable_content_binary(&self) -> Option<BinaryExecOutputSupplier> {
        match self {
            CapturableExecutable::TextFromExecutableUri(uri, _, _) => {
                Some(executable_content_binary(uri))
            }
            CapturableExecutable::TextFromDenoTaskShellCmd(_uri, src, _, _) => {
                Some(deno_task_shell_content_binary(src))
            }
            CapturableExecutable::RequestedButNoNature(src, _) => {
                Some(executable_content_binary(src))
            }
            CapturableExecutable::RequestedButNotExecutable(_) => None,
        }
    }
}

const DEFAULT_CAPTURE_EXEC_REGEX_PATTERN: &str = r"surveilr\[(?P<nature>[^\]]*)\]";
const DEFAULT_CAPTURE_SQL_EXEC_REGEX_PATTERN: &str = r"surveilr-SQL";

pub trait CapturableExecutableSupplier {
    fn capturable_executable(&self) -> Option<CapturableExecutable>;
}

#[derive(Debug, Clone)]
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

    // check if URI is executable based only on the filename pattern
    pub fn uri_capturable_executable(&self, uri: &str) -> Option<CapturableExecutable> {
        let mut ce: Option<CapturableExecutable> = None;

        if self.capturable_sql_set.is_match(uri) {
            ce = Some(CapturableExecutable::TextFromExecutableUri(
                uri.to_string(),
                String::from("surveilr-SQL"),
                true,
            ));
        } else {
            for re in self.capturable_regexs.iter() {
                if let Some(caps) = re.captures(uri) {
                    if let Some(nature) = caps.name("nature") {
                        ce = Some(CapturableExecutable::TextFromExecutableUri(
                            uri.to_string(),
                            String::from(nature.as_str()),
                            false,
                        ));
                        break;
                    } else {
                        ce = Some(CapturableExecutable::RequestedButNoNature(
                            uri.to_string(),
                            re.clone(),
                        ));
                        break;
                    }
                }
            }
        }
        ce
    }

    // check if URI is executable based the filename pattern first, then physical FS validation of execute permission
    pub fn path_capturable_executable(
        &self,
        path: &std::path::Path,
    ) -> Option<CapturableExecutable> {
        let uri_ce = self.uri_capturable_executable(path.to_str().unwrap());
        if uri_ce.is_some() {
            if path.is_executable() {
                return uri_ce;
            } else {
                return Some(CapturableExecutable::RequestedButNotExecutable(
                    path.to_string_lossy().to_string(),
                ));
            }
        }
        None
    }

    // check if URI is executable based the filename pattern first, then physical FS validation of execute permission
    pub fn smart_path_capturable_executable(&self, uri: &str) -> Option<CapturableExecutable> {
        self.path_capturable_executable(std::path::Path::new(uri))
    }
}
