use anyhow::Context;
use regex::Regex;
use serde_json::json;

use super::CapturableExecCommands;
use crate::fsresource::*;
use crate::resource::*;

// Implement methods for `CapturableExecCommands`, ensure that whether the commands
// are called from CLI or natively within Rust, all the calls remain ergonomic.
impl CapturableExecCommands {
    pub fn execute(
        &self,
        cli: &super::Cli,
        _args: &super::CapturableExecArgs,
    ) -> anyhow::Result<()> {
        match self {
            CapturableExecCommands::Ls {
                root_path,
                ignore_entry,
                capture_exec,
                captured_exec_sql,
            } => self.ls(
                cli,
                root_path,
                capture_exec,
                captured_exec_sql,
                ignore_entry,
            ),
        }
    }

    fn ls(
        &self,
        _cli: &super::Cli,
        root_paths: &[String],
        capture_exec: &[Regex],
        captured_exec_sql: &[Regex],
        ignore_entries: &[Regex],
    ) -> anyhow::Result<()> {
        let walker = FileSysResourcesWalker::new(
            root_paths,
            ignore_entries,
            &[],
            capture_exec,
            captured_exec_sql,
        )
        .with_context(|| "[CapturableExecCommands::ls] unable to create fs walker")?;

        let mut found: Vec<Vec<String>> = vec![];
        for resource_result in walker.walk_resources_iter() {
            match resource_result {
                Ok((dir_entry, ur)) => {
                    if let crate::resource::UniformResource::CapturableExec(cer) = ur {
                        match &cer.executable.capturable_executable {
                            Some(capturable_executable) => match capturable_executable {
                                CapturableExecutable::Text(nature, is_batched_sql) => {
                                    match cer.executable.capturable_exec_text_supplier.as_ref() {
                                        Some(capturable_supplier) => {
                                            match capturable_supplier(Some(
                                                serde_json::to_string_pretty(&json!(
                                                    // simulated STDIN because some scripts expect it
                                                    r#"{ "surveilr-fs-walk": {} }"#
                                                ))
                                                .unwrap(),
                                            )) {
                                                Ok((capture_src, exit_status, _stderr)) => {
                                                    if matches!(
                                                        exit_status,
                                                        subprocess::ExitStatus::Exited(0)
                                                    ) {
                                                        let hash = String::from(
                                                            capture_src.content_digest_hash(),
                                                        );
                                                        if *is_batched_sql {
                                                            found.push(vec![
                                                                dir_entry
                                                                    .path()
                                                                    .to_string_lossy()
                                                                    .to_string(),
                                                                format!("{:?}", exit_status),
                                                                String::from("batched SQL"),
                                                                hash,
                                                            ])
                                                        } else {
                                                            found.push(vec![
                                                                dir_entry
                                                                    .path()
                                                                    .to_string_lossy()
                                                                    .to_string(),
                                                                format!("{:?}", exit_status),
                                                                nature.clone(),
                                                                hash,
                                                            ])
                                                        }
                                                    } else {
                                                        found.push(vec![
                                                            dir_entry
                                                                .path()
                                                                .to_string_lossy()
                                                                .to_string(),
                                                            format!("{:?}", exit_status),
                                                            nature.clone(),
                                                            String::from(""),
                                                        ]);
                                                    }
                                                }
                                                Err(err) => {
                                                    found.push(vec![
                                                        dir_entry
                                                            .path()
                                                            .to_string_lossy()
                                                            .to_string(),
                                                        format!("{:?}", err),
                                                        nature.clone(),
                                                        String::from(""),
                                                    ]);
                                                }
                                            }
                                        }
                                        None => {
                                            found.push(vec![
                                                dir_entry.path().to_string_lossy().to_string(),
                                                String::from("No CE Supplier"),
                                                nature.clone(),
                                                String::from(""),
                                            ]);
                                        }
                                    }
                                }
                                CapturableExecutable::RequestedButNoNature(re) => {
                                    found.push(vec![
                                        dir_entry.path().to_string_lossy().to_string(),
                                        String::from("No CE Nature in reg ex"),
                                        re.to_string(),
                                        String::from(""),
                                    ]);
                                }
                                CapturableExecutable::RequestedButNotExecutable => {
                                    found.push(vec![
                                        dir_entry.path().to_string_lossy().to_string(),
                                        String::from("Executable Permission Not Set"),
                                        String::from(""),
                                        String::from(""),
                                    ]);
                                }
                            },
                            None => {
                                found.push(vec![
                                    dir_entry.path().to_string_lossy().to_string(),
                                    String::from("Executable Permission Not Set"),
                                    String::from(""),
                                    String::from(""),
                                ]);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error processing a resource: {}", e);
                }
            }
        }

        if !found.is_empty() {
            println!(
                "{}",
                crate::format::format_table(&["Executable", "Status", "Nature", "Hash"], &found)
            );
        }

        Ok(())
    }
}
