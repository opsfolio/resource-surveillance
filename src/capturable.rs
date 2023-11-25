use is_executable::IsExecutable;

use regex::{Regex, RegexSet};

const DEFAULT_CAPTURE_EXEC_REGEX_PATTERN: &str = r"surveilr\[(?P<nature>[^\]]*)\]";
const DEFAULT_CAPTURE_SQL_EXEC_REGEX_PATTERN: &str = r"surveilr-SQL";

#[derive(Debug, Clone)]
pub enum CapturableExecutable {
    Text(String, bool),
    RequestedButNoNature(Regex),
    RequestedButNotExecutable,
}

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
