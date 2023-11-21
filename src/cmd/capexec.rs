use std::collections::HashMap;

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
                markdown,
            } => {
                if *markdown {
                    self.ls_markdown(
                        cli,
                        root_path,
                        capture_exec,
                        captured_exec_sql,
                        ignore_entry,
                    )
                } else {
                    self.ls_table(
                        cli,
                        root_path,
                        capture_exec,
                        captured_exec_sql,
                        ignore_entry,
                    )
                }
            }
        }
    }

    fn ls_table(
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
            &HashMap::new(),
        )
        .with_context(|| "[CapturableExecCommands::ls] unable to create fs walker")?;

        let mut found: Vec<Vec<String>> = vec![];
        for resource_result in walker.walk_resources_iter() {
            match resource_result {
                Ok((dir_entry, ur)) => {
                    let dir_entry_path = dir_entry.path().to_string_lossy().to_string();

                    if let crate::resource::UniformResource::CapturableExec(cer) = ur {
                        match &cer.executable.capturable_executable {
                            Some(capturable_executable) => match capturable_executable {
                                CapturableExecutable::Text(nature, is_batched_sql) => {
                                    if *is_batched_sql {
                                        found.push(vec![
                                            dir_entry_path,
                                            String::from("batched SQL"),
                                            String::from(""),
                                        ])
                                    } else {
                                        found.push(vec![
                                            dir_entry_path,
                                            nature.clone(),
                                            String::from(""),
                                        ])
                                    }
                                }
                                CapturableExecutable::RequestedButNoNature(re) => {
                                    found.push(vec![
                                        dir_entry_path,
                                        String::from("No CE Nature in reg ex"),
                                        format!("{}", re.to_string()),
                                    ]);
                                }
                                CapturableExecutable::RequestedButNotExecutable => {
                                    found.push(vec![
                                        dir_entry_path,
                                        String::from("Executable Permission Not Set"),
                                        String::from("chmod +x required"),
                                    ]);
                                }
                            },
                            None => {
                                found.push(vec![
                                    dir_entry_path,
                                    String::from(
                                        "cer.executable.capturable_executable returned None",
                                    ),
                                    String::from("needs investigation"),
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
                crate::format::format_table(&["Executable", "Nature", "Issue"], &found)
            );
        }

        Ok(())
    }

    fn ls_markdown(
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
            &HashMap::new(),
        )
        .with_context(|| "[CapturableExecCommands::ls] unable to create fs walker")?;

        let mut markdown: Vec<String> = vec!["# `surveilr` Capturable Executables\n\n".to_string()];

        markdown.push("Root Paths\n".to_string());
        markdown.push(
            root_paths
                .iter()
                .fold("".to_string(), |_acc, x| format!("- `{}`\n", x)),
        );

        markdown.push("\nCapturable Executables RegExes\n".to_string());
        markdown.push(
            capture_exec
                .iter()
                .fold("".to_string(), |_acc, x| format!("- `{}`\n", x)),
        );

        markdown.push("\nCapturable Executables Batched SQL RegExes\n".to_string());
        markdown.push(
            captured_exec_sql
                .iter()
                .fold("".to_string(), |_acc, x| format!("- `{}`\n", x)),
        );

        markdown.push("\nIgnore Entries\n".to_string());
        markdown.push(
            ignore_entries
                .iter()
                .fold("".to_string(), |_acc, x| format!("- `{}`\n", x)),
        );
        markdown.push("\n".to_string());

        for resource_result in walker.walk_resources_iter() {
            match resource_result {
                Ok((dir_entry, ur)) => {
                    if let crate::resource::UniformResource::CapturableExec(cer) = ur {
                        markdown.push(format!(
                            "## {}\n\n",
                            dir_entry.file_name().to_string_lossy()
                        ));
                        markdown.push(format!("- `{}`\n", dir_entry.path().to_string_lossy()));

                        match &cer.executable.capturable_executable {
                            Some(capturable_executable) => match capturable_executable {
                                CapturableExecutable::Text(nature, is_batched_sql) => {
                                    markdown.push(format!("- Nature: `{}`\n", nature));
                                    markdown
                                        .push(format!("- Batched SQL?: `{}`\n", is_batched_sql));

                                    match cer.executable.capturable_exec_text_supplier.as_ref() {
                                        Some(capturable_supplier) => {
                                            let synthetic_stdin = json!({
                                                "surveilr-fs-walk": {
                                                    "args": { "state_db_fs_path": "synthetic" },
                                                    "env": { "current_dir": std::env::current_dir().unwrap().to_string_lossy() },
                                                    "behavior": {},
                                                    "device": { "device_id": "synthetic" },
                                                    "session": {
                                                        "walk-session-id":  "synthetic",
                                                        "walk-path-id":  "synthetic",
                                                        "entry": { "path": dir_entry.path() },
                                                    },
                                                }
                                            });
                                            let synthetic_stdin =
                                                serde_json::to_string_pretty(&synthetic_stdin)
                                                    .unwrap();

                                            match capturable_supplier(Some(synthetic_stdin.clone()))
                                            {
                                                Ok((capture_src, exit_status, stderr)) => {
                                                    markdown
                                                        .push(format!("- `{:?}`\n\n", exit_status));

                                                    markdown.push("\nSTDOUT\n".to_string());
                                                    markdown.push(format!("```{}\n", nature));
                                                    markdown.push(format!(
                                                        "{}\n",
                                                        capture_src.content_text()
                                                    ));
                                                    markdown.push("```\n".to_string());
                                                    markdown.push(format!(
                                                        "> {}\n\n",
                                                        capture_src.content_digest_hash()
                                                    ));

                                                    if let Some(stderr) = stderr {
                                                        markdown.push("STDERR\n".to_string());
                                                        markdown.push("```\n".to_string());
                                                        markdown.push(format!("{}\n", stderr));
                                                        markdown.push("```\n\n".to_string());
                                                    }

                                                    markdown.push(
                                                        "Synthetic STDIN (for testing the execution)\n"
                                                            .to_string(),
                                                    );
                                                    markdown.push("```json\n".to_string());
                                                    markdown.push(format!(
                                                        "{}\n",
                                                        synthetic_stdin.clone()
                                                    ));
                                                    markdown.push("```\n".to_string());
                                                }
                                                Err(err) => {
                                                    markdown.push("\nRust Error\n".to_string());
                                                    markdown.push("```\n".to_string());
                                                    markdown.push(format!("{:?}\n", err));
                                                    markdown.push("```\n".to_string());
                                                }
                                            }
                                        }
                                        None => {
                                            markdown.push(format!("- {}\n", "No CE Supplier"));
                                        }
                                    }
                                }
                                CapturableExecutable::RequestedButNoNature(re) => {
                                    markdown
                                        .push(format!("- {} {}\n", "No CE Nature in reg ex", re));
                                }
                                CapturableExecutable::RequestedButNotExecutable => {
                                    markdown
                                        .push(format!("- {}\n", "Executable Permission Not Set"));
                                }
                            },
                            None => {
                                markdown.push(format!(
                                    "- {}\n",
                                    "cer.executable.capturable_executable returned None"
                                ));
                            }
                        }
                    }
                }
                Err(e) => {
                    markdown.push(format!("\nRust Error\n```\n{}\n```", e));
                }
            }
        }

        if !markdown.is_empty() {
            println!("{}", markdown.join(""));
        }

        Ok(())
    }
}
