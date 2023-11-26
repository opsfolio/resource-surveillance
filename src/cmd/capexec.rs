use std::collections::HashMap;

use anyhow::Context;
use regex::Regex;
use serde_json::json;

use super::CapturableExecCommands;
use crate::capturable::*;
use crate::fsresource::*;
use crate::subprocess::CapturableExecutableStdIn;

// Implement methods for `CapturableExecCommands`, ensure that whether the commands
// are called from CLI or natively within Rust, all the calls remain ergonomic.
impl CapturableExecCommands {
    pub fn execute(
        &self,
        cli: &super::Cli,
        args: &super::CapturableExecArgs,
    ) -> anyhow::Result<()> {
        match self {
            CapturableExecCommands::Ls {
                root_fs_path: root_path,
                ignore_fs_entry: ignore_entry,
                capture_fs_exec: capture_exec,
                captured_fs_exec_sql: captured_exec_sql,
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
            CapturableExecCommands::Test {
                fs_path,
                capture_fs_exec: capture_exec,
                captured_fs_exec_sql: captured_exec_sql,
            } => self.test_fs_path(cli, args, fs_path, capture_exec, captured_exec_sql),
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
                                CapturableExecutable::TextFromExecutableUri(
                                    _uri,
                                    nature,
                                    is_batched_sql,
                                ) => {
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
                                CapturableExecutable::TextFromDenoTaskShellCmd(
                                    _uri,
                                    _src,
                                    nature,
                                    is_batched_sql,
                                ) => {
                                    if *is_batched_sql {
                                        found.push(vec![
                                            dir_entry_path,
                                            String::from("batched SQL"),
                                            String::from("Should never appear in this list since Deno Tasks are stored in memory or database"),
                                        ])
                                    } else {
                                        found.push(vec![
                                            dir_entry_path,
                                            nature.clone(),
                                            String::from("Should never appear in this list since Deno Tasks are stored in memory or database"),
                                        ])
                                    }
                                }
                                CapturableExecutable::RequestedButNoNature(_src, re) => {
                                    found.push(vec![
                                        dir_entry_path,
                                        String::from("No CE Nature in reg ex"),
                                        format!("{}", re.to_string()),
                                    ]);
                                }
                                CapturableExecutable::RequestedButNotExecutable(_src) => {
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
                                CapturableExecutable::TextFromExecutableUri(
                                    _,
                                    nature,
                                    is_batched_sql,
                                )
                                | CapturableExecutable::TextFromDenoTaskShellCmd(
                                    _,
                                    _,
                                    nature,
                                    is_batched_sql,
                                ) => {
                                    markdown.push(format!("- Nature: `{}`\n", nature));
                                    markdown
                                        .push(format!("- Batched SQL?: `{}`\n", is_batched_sql));

                                    match cer.executable.capturable_exec_text_supplier.as_ref() {
                                        Some(capturable_supplier) => {
                                            let synthetic_stdin = json!({
                                                "surveilr-ingest": {
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
                                CapturableExecutable::RequestedButNoNature(_src, re) => {
                                    markdown
                                        .push(format!("- {} {}\n", "No CE Nature in reg ex", re));
                                }
                                CapturableExecutable::RequestedButNotExecutable(_src) => {
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

    fn test_fs_path(
        &self,
        cli: &super::Cli,
        args: &super::CapturableExecArgs,
        fs_path: &String,
        capture_exec: &[Regex],
        captured_exec_sql: &[Regex],
    ) -> anyhow::Result<()> {
        let cerr = CapturableExecutableRegexRules::new(Some(capture_exec), Some(captured_exec_sql))
            .with_context(|| "unable to create CapturableExecutableRegexRules")?;
        match cerr.capturable_executable(std::path::Path::new(fs_path)) {
            Some(ce) => {
                let unknown_nature = "UNKNOWN_NATURE".to_string();
                // pass in synthetic JSON into STDIN since some scripts may try to consume stdin
                let stdin = CapturableExecutableStdIn::from_json(serde_json::json!({
                    "cli": cli,
                    "args": args
                }));
                let (src, nature, is_batch_sql) = match &ce {
                    CapturableExecutable::TextFromExecutableUri(uri, nature, is_batch_sql) => {
                        (uri, nature, is_batch_sql)
                    }
                    CapturableExecutable::TextFromDenoTaskShellCmd(
                        _uri,
                        src,
                        nature,
                        is_batch_sql,
                    ) => (src, nature, is_batch_sql),
                    CapturableExecutable::RequestedButNoNature(uri, _) => {
                        (uri, &unknown_nature, &false)
                    }
                    CapturableExecutable::RequestedButNotExecutable(uri) => {
                        (uri, &unknown_nature, &false)
                    }
                };
                println!("src: {}", src);
                println!("nature: {} (is batch SQL: {})", nature, is_batch_sql);
                let mut emitted = 0;

                if nature == "json" {
                    match ce.executed_result_as_json(stdin.clone()) {
                        Ok((stdout_json, _nature, _is_batch_sql)) => {
                            println!("{}", serde_json::to_string_pretty(&stdout_json).unwrap())
                        }
                        Err(error_json) => {
                            eprintln!("{}", serde_json::to_string_pretty(&error_json).unwrap())
                        }
                    }
                    emitted += 1;
                }

                if nature == "surveilr-SQL" {
                    match ce.executed_result_as_sql(stdin.clone()) {
                        Ok((stdout_sql, _nature)) => {
                            println!("{}", stdout_sql)
                        }
                        Err(error_json) => {
                            eprintln!("{}", serde_json::to_string_pretty(&error_json).unwrap())
                        }
                    }
                    emitted += 1;
                }

                if emitted == 0 {
                    match ce.executed_result_as_text(stdin.clone()) {
                        Ok((stdout_text, _nature, _is_batch_sql)) => {
                            println!("{}", stdout_text)
                        }
                        Err(error_json) => {
                            eprintln!("{}", serde_json::to_string_pretty(&error_json).unwrap())
                        }
                    }
                }
            }
            None => println!(
                "Did not match capturable executable regex rules: {:?}",
                cerr
            ),
        }

        Ok(())
    }
}
