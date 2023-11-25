use is_executable::IsExecutable;
use regex::{Regex, RegexSet};

use crate::subprocess::*;

#[derive(Debug, Clone)]
pub enum CapturableExecutable {
    Text(String, String, bool),
    RequestedButNoNature(String, Regex),
    RequestedButNotExecutable(String),
}

impl CapturableExecutable {
    pub fn executed_result_as_text(
        &self,
        std_in: CapturableExecutableStdIn,
    ) -> anyhow::Result<(String, String, bool), serde_json::Value> {
        match self {
            CapturableExecutable::Text(src, nature, is_batched_sql) => {
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
                                "issue": "[CapturableExecutable::Text.executed_text] invalid exit status",
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
                        "issue": "[CapturableExecutable::Text.executed_text] execution error",
                        "rust-err": format!("{:?}", err),
                        "nature": nature,
                    })),
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
            CapturableExecutable::Text(src, nature, is_batched_sql) => {
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
                                    "issue": "[CapturableExecutable::Text.executed_json] unable to deserialize JSON",
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
                                "issue": "[CapturableExecutable::Text.executed_json] invalid exit status",
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
                        "issue": "[CapturableExecutable::Text.executed_json] execution error",
                        "rust-err": format!("{:?}", err),
                        "nature": nature,
                        "is-batched-sql": is_batched_sql,
                    })),
                }
            }
            CapturableExecutable::RequestedButNoNature(src, regex) => Err(serde_json::json!({
                "src": src,
                "issue": "[CapturableExecutable::RequestedButNoNature.executed_json] unable to determine nature",
                "remediation": "make sure that the regular expression has a `nature` named capture group",
                "regex": format!("{:?}", regex),
            })),
            CapturableExecutable::RequestedButNotExecutable(src) => Err(serde_json::json!({
                "src": src,
                "issue": "[CapturableExecutable::RequestedButNotExecutable.executed_json] executable permissions not set",
                "remediation": "make sure that script has executable permissions set",
            })),
        }
    }

    pub fn executed_result_as_sql(
        &self,
        std_in: CapturableExecutableStdIn,
    ) -> anyhow::Result<(String, String), serde_json::Value> {
        match self {
            CapturableExecutable::Text(src, nature, is_batched_sql) => {
                if *is_batched_sql {
                    match execution_result_text(src, std_in) {
                        Ok((stdout, status, stderr)) => {
                            if status.success() {
                                Ok((String::from(stdout.content_text()), nature.clone()))
                            } else {
                                Err(serde_json::json!({
                                    "src": src,
                                    "issue": "[CapturableExecutable::Text.executed_sql] invalid exit status",
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
                            "issue": "[CapturableExecutable::Text.executed_sql] execution error",
                            "rust-err": format!("{:?}", err),
                            "nature": nature,
                        })),
                    }
                } else {
                    Err(serde_json::json!({
                        "src": src,
                        "issue": "[CapturableExecutable::Text.executed_sql] is not classified as batch SQL",
                        "nature": nature,
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

    pub fn executable_content_text(&self) -> Option<TextExecOutputSupplier> {
        match self {
            CapturableExecutable::Text(src, _, _) => Some(executable_content_text(src)),
            CapturableExecutable::RequestedButNoNature(src, _) => {
                Some(executable_content_text(src))
            }
            CapturableExecutable::RequestedButNotExecutable(_) => None,
        }
    }

    pub fn executable_content_binary(&self) -> Option<BinaryExecOutputSupplier> {
        match self {
            CapturableExecutable::Text(src, _, _) => Some(executable_content_binary(src)),
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

    pub fn capturable_executable(&self, path: &std::path::Path) -> Option<CapturableExecutable> {
        let mut ce: Option<CapturableExecutable> = None;
        let haystack: &str = path.to_str().unwrap();

        if self.capturable_sql_set.is_match(haystack) {
            ce = Some(CapturableExecutable::Text(
                haystack.to_string(),
                String::from("surveilr-SQL"),
                true,
            ));
        } else {
            for re in self.capturable_regexs.iter() {
                if let Some(caps) = re.captures(haystack) {
                    if let Some(nature) = caps.name("nature") {
                        ce = Some(CapturableExecutable::Text(
                            haystack.to_string(),
                            String::from(nature.as_str()),
                            false,
                        ));
                        break;
                    } else {
                        ce = Some(CapturableExecutable::RequestedButNoNature(
                            haystack.to_string(),
                            re.clone(),
                        ));
                        break;
                    }
                }
            }
        }
        if ce.is_some() {
            if path.is_executable() {
                return ce;
            } else {
                return Some(CapturableExecutable::RequestedButNotExecutable(
                    haystack.to_string(),
                ));
            }
        }
        None
    }
}
