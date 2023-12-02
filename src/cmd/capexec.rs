use std::collections::HashMap;
use std::env;

use anyhow::Context;
use regex::Regex;
use serde_json::json;

use super::CapturableExecCommands;
use super::CapturableExecTestCommands;
use crate::resource::*;
use crate::shell::*;

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
            CapturableExecCommands::Test(test_args) => {
                test_args.command.execute(cli, args, test_args)
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
        let resources = ResourcesCollection::from_smart_ignore(
            root_paths,
            &ResourcesCollectionOptions {
                ingest_content_regexs: vec![],
                ignore_paths_regexs: ignore_entries.to_vec(),
                capturable_executables_regexs: capture_exec.to_vec(),
                captured_exec_sql_regexs: captured_exec_sql.to_vec(),
                nature_bind: HashMap::default(),
            },
            super::DEFAULT_IGNORE_GLOBS_CONF_FILE,
            false,
        );

        let mut found: Vec<Vec<String>> = vec![];
        for resource_result in resources.uniform_resources() {
            match resource_result {
                Ok(ur) => {
                    let path = ur.uri().clone();
                    let mut relative_path = ur
                        .uri()
                        .strip_prefix(
                            &env::current_dir()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                        )
                        .unwrap_or(ur.uri())
                        .to_string();
                    if path != relative_path {
                        // if we computed a relative path, the strip_prefix would remove the current_dir but leave /
                        relative_path.insert(0, '.')
                    }

                    if let crate::resource::UniformResource::CapturableExec(cer) = ur {
                        match &cer.executable {
                            CapturableExecutable::UriShellExecutive(
                                _executive,
                                _uri,
                                nature,
                                is_batched_sql,
                            ) => {
                                if *is_batched_sql {
                                    found.push(vec![
                                        relative_path,
                                        String::from("batched SQL"),
                                        String::from(""),
                                    ])
                                } else {
                                    found.push(vec![
                                        relative_path,
                                        nature.clone(),
                                        String::from(""),
                                    ])
                                }
                            }
                            CapturableExecutable::RequestedButNoNature(_src, re) => {
                                found.push(vec![
                                    relative_path,
                                    String::from("No CE Nature in reg ex"),
                                    format!("{}", re.to_string()),
                                ]);
                            }
                            CapturableExecutable::RequestedButNotExecutable(_src) => {
                                found.push(vec![
                                    relative_path,
                                    String::from("Executable Permission Not Set"),
                                    String::from("chmod +x required"),
                                ]);
                            }
                        }
                    }
                }
                Err(_) => {
                    // unable to determine the kind of file, so it's not a capturable executable
                }
            }
        }

        if !found.is_empty() {
            println!(
                "{}",
                crate::format::as_ascii_table(&["Executable", "Nature", "Issue"], &found)
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
        let resources = ResourcesCollection::from_smart_ignore(
            root_paths,
            &ResourcesCollectionOptions {
                ingest_content_regexs: vec![],
                ignore_paths_regexs: ignore_entries.to_vec(),
                capturable_executables_regexs: capture_exec.to_vec(),
                captured_exec_sql_regexs: captured_exec_sql.to_vec(),
                nature_bind: HashMap::default(),
            },
            super::DEFAULT_IGNORE_GLOBS_CONF_FILE,
            false,
        );

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

        for resource_result in resources.uniform_resources() {
            match resource_result {
                Ok(ur) => {
                    if let crate::resource::UniformResource::CapturableExec(cer) = &ur {
                        let path = ur.uri().clone();
                        markdown.push(format!("## {}\n\n", path)); // TODO: replace with just the filename
                                                                   // markdown.push(format!("- `{}`\n", path));

                        match &cer.executable {
                            CapturableExecutable::UriShellExecutive(
                                executive,
                                _,
                                nature,
                                is_batched_sql,
                            ) => {
                                markdown.push(format!("- Nature: `{}`\n", nature));
                                markdown.push(format!("- Batched SQL?: `{}`\n", is_batched_sql));

                                let synthetic_stdin = json!({
                                    "surveilr-ingest": {
                                        "args": { "state_db_fs_path": "synthetic" },
                                        "env": { "current_dir": std::env::current_dir().unwrap().to_string_lossy() },
                                        "behavior": {},
                                        "device": { "device_id": "synthetic" },
                                        "session": {
                                            "walk-session-id":  "synthetic",
                                            "walk-path-id":  "synthetic",
                                            "entry": { "path": path },
                                        },
                                    }
                                });

                                match executive.execute(ShellStdIn::Json(synthetic_stdin.clone())) {
                                    Ok(shell_result) => {
                                        markdown.push(format!("- `{:?}`\n\n", shell_result.status));

                                        markdown.push("\nSTDOUT\n".to_string());
                                        markdown.push(format!("```{}\n", nature));
                                        markdown.push(format!("{}\n", shell_result.stdout));
                                        markdown.push("```\n".to_string());
                                        markdown
                                            .push(format!("> {}\n\n", shell_result.stdout_hash()));

                                        if !shell_result.stderr.is_empty() {
                                            markdown.push("STDERR\n".to_string());
                                            markdown.push("```\n".to_string());
                                            markdown.push(format!("{}\n", shell_result.stderr));
                                            markdown.push("```\n\n".to_string());
                                        }

                                        markdown.push(
                                            "Synthetic STDIN (for testing the execution)\n"
                                                .to_string(),
                                        );
                                        markdown.push("```json\n".to_string());
                                        markdown.push(format!("{}\n", synthetic_stdin.clone()));
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
                            CapturableExecutable::RequestedButNoNature(_src, re) => {
                                markdown.push(format!("- {} {}\n", "No CE Nature in reg ex", re));
                            }
                            CapturableExecutable::RequestedButNotExecutable(_src) => {
                                markdown.push(format!("- {}\n", "Executable Permission Not Set"));
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

// Implement methods for `CapturableExecCommands`, ensure that whether the commands
// are called from CLI or natively within Rust, all the calls remain ergonomic.
impl CapturableExecTestCommands {
    pub fn execute(
        &self,
        cli: &super::Cli,
        parent_args: &super::CapturableExecArgs,
        cmd_args: &super::CapturableExecTestArgs,
    ) -> anyhow::Result<()> {
        match self {
            CapturableExecTestCommands::File {
                fs_path,
                capture_fs_exec: capture_exec,
                captured_fs_exec_sql: captured_exec_sql,
            } => self.test_fs_path(
                cli,
                parent_args,
                cmd_args,
                fs_path,
                capture_exec,
                captured_exec_sql,
            ),
            CapturableExecTestCommands::Task {
                task,
                cwd,
                stdout_only,
                nature,
            } => self.task(cli, task, nature, cwd.as_ref(), *stdout_only),
        }
    }

    fn test_fs_path(
        &self,
        cli: &super::Cli,
        _parent_args: &super::CapturableExecArgs,
        cmd_args: &super::CapturableExecTestArgs,
        fs_path: &String,
        capture_exec: &[Regex],
        captured_exec_sql: &[Regex],
    ) -> anyhow::Result<()> {
        let cerr = CapturableExecutableRegexRules::new(Some(capture_exec), Some(captured_exec_sql))
            .with_context(|| "unable to create CapturableExecutableRegexRules")?;
        match cerr.path_capturable_executable(std::path::Path::new(fs_path)) {
            Some(ce) => {
                let unknown_nature = "UNKNOWN_NATURE".to_string();
                // pass in synthetic JSON into STDIN since some scripts may try to consume stdin
                let stdin = ShellStdIn::Json(serde_json::json!({
                    "cli": cli,
                    "args": cmd_args
                }));
                let (src, nature, is_batch_sql) = match &ce {
                    CapturableExecutable::UriShellExecutive(_, uri, nature, is_batch_sql) => {
                        (uri.clone(), nature, is_batch_sql)
                    }
                    CapturableExecutable::RequestedButNoNature(uri, _) => {
                        (uri.clone(), &unknown_nature, &false)
                    }
                    CapturableExecutable::RequestedButNotExecutable(uri) => {
                        (uri.clone(), &unknown_nature, &false)
                    }
                };
                println!("src: {}", src);
                println!("nature: {} (is batch SQL: {})", nature, is_batch_sql);
                let mut emitted = 0;

                if nature == "json" {
                    println!("{:?}", ce.executed_result_as_json(stdin.clone()));
                    emitted += 1;
                }

                if nature == "surveilr-SQL" {
                    println!("{:?}", ce.executed_result_as_sql(stdin.clone()));
                    emitted += 1;
                }

                if emitted == 0 {
                    match ce.executed_result_as_text(stdin) {
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

    fn task(
        &self,
        cli: &super::Cli,
        command: &str,
        nature: &str,
        _cwd: Option<&String>,
        _stdout_only: bool,
    ) -> anyhow::Result<()> {
        if cli.debug > 0 {
            println!("{:?}", command);
        }

        let stdin = crate::shell::ShellStdIn::None;
        let ce = CapturableExecutable::UriShellExecutive(
            Box::new(DenoTaskShellExecutive::new(command.to_owned(), None)),
            format!("cli://capturable-exec/test/task/{}", command),
            nature.to_owned(),
            false,
        );

        match nature {
            "json" | "text/json" | "application/json" => match ce.executed_result_as_json(stdin) {
                Ok((json_value, _nature, _is_sql_exec)) => {
                    print!("{}", serde_json::to_string_pretty(&json_value).unwrap());
                }
                Err(err) => {
                    print!("{:?}", err);
                }
            },
            _ => match ce.executed_result_as_text(stdin) {
                Ok((stdout, _nature, _is_sql_exec)) => {
                    print!("{stdout}");
                }
                Err(err) => {
                    print!("{:?}", err);
                }
            },
        }

        Ok(())
    }
}
