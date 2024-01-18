use std::collections::HashMap;
use std::env;

use autometrics::autometrics;
use cmd::CapturableExecArgs;
use cmd::CapturableExecCommands;
use cmd::CapturableExecTestArgs;
use cmd::CapturableExecTestCommands;
use resource::*;
use resource::shell::ShellStdIn;
use serde_json::json;
use tracing::debug;
use tracing::error;
use tracing::info;


// Implement methods for `CapturableExecCommands`, ensure that whether the commands
// are called from CLI or natively within Rust, all the calls remain ergonomic.
#[derive(Debug, Default)]
pub struct CapturableExec {}

impl CapturableExec {

    #[autometrics]
    pub fn execute(
        &self,
        cli: &super::Cli,
        args: &CapturableExecArgs,
    ) -> anyhow::Result<()> {
        match &args.command {
            CapturableExecCommands::Ls {
                root_fs_path: root_path,
                markdown,
            } => {
                if *markdown {
                    self.ls_markdown(cli, root_path)
                } else {
                    self.ls_table(cli, root_path)
                }
            }
            CapturableExecCommands::Test(test_args) => {
                CapturableExecTest::new().execute(cli, args, test_args)
            }
        }
    }

    fn ls_table(&self, _cli: &super::Cli, root_paths: &[String]) -> anyhow::Result<()> {
        let resources =
            ResourcesCollection::from_smart_ignore(root_paths, &Default::default(), None, false);

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

                    if let resource::UniformResource::CapturableExec(cer) = ur {
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
            info!(
                "{}",
                common::format::as_ascii_table(&["Executable", "Nature", "Issue"], &found)
            );
        }

        Ok(())
    }

    fn ls_markdown(&self, _cli: &super::Cli, root_paths: &[String]) -> anyhow::Result<()> {
        let classifier: EncounterableResourcePathClassifier = Default::default();
        let resources =
            ResourcesCollection::from_smart_ignore(root_paths, &classifier, None, false);

        let mut markdown: Vec<String> = vec!["# `surveilr` Capturable Executables\n\n".to_string()];

        markdown.push("Root Paths\n".to_string());
        markdown.push(
            root_paths
                .iter()
                .fold("".to_string(), |_acc, x| format!("- `{}`\n", x)),
        );

        markdown.push("\nCapturable Executables RegExes\n".to_string());
        markdown.push(
            classifier
                .flaggables
                .iter()
                .filter(|f| {
                    f.flags
                        .contains(EncounterableResourceFlags::CAPTURABLE_EXECUTABLE)
                })
                .map(|f| f.regex.clone())
                .fold("".to_string(), |_acc, x| format!("- `{}`\n", x)),
        );

        markdown.push("\nCapturable Executables Batched SQL RegExes\n".to_string());
        markdown.push(
            classifier
                .flaggables
                .iter()
                .filter(|f| f.flags.contains(EncounterableResourceFlags::CAPTURABLE_SQL))
                .map(|f| f.regex.clone())
                .fold("".to_string(), |_acc, x| format!("- `{}`\n", x)),
        );

        markdown.push("\nIgnore Entries\n".to_string());
        markdown.push(
            classifier
                .flaggables
                .iter()
                .filter(|f| {
                    f.flags
                        .contains(EncounterableResourceFlags::IGNORE_RESOURCE)
                })
                .map(|f| f.regex.clone())
                .fold("".to_string(), |_acc, x| format!("- `{}`\n", x)),
        );
        markdown.push("\n".to_string());

        for resource_result in resources.uniform_resources() {
            match resource_result {
                Ok(ur) => {
                    if let UniformResource::CapturableExec(cer) = &ur {
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
            info!("{}", markdown.join(""));
        }

        Ok(())
    }
}

struct CapturableExecTest {}

// Implement methods for `CapturableExecCommands`, ensure that whether the commands
// are called from CLI or natively within Rust, all the calls remain ergonomic.
impl CapturableExecTest {

    pub fn new() -> CapturableExecTest {
        CapturableExecTest {}
    }

    // #[autometrics]
    pub fn execute(
        &self,
        cli: &super::Cli,
        parent_args: &CapturableExecArgs,
        cmd_args: &CapturableExecTestArgs,
    ) -> anyhow::Result<()> {
        match &cmd_args.command {
            CapturableExecTestCommands::File { fs_path } => {
                self.test_fs_path(cli, parent_args, cmd_args, fs_path)
            }
            CapturableExecTestCommands::Task { stdin, task, cwd } => {
                self.task(*stdin, task, cwd.as_ref())
            }
        }
    }

    fn test_fs_path(
        &self,
        cli: &super::Cli,
        _parent_args: &CapturableExecArgs,
        cmd_args: &CapturableExecTestArgs,
        fs_path: &str,
    ) -> anyhow::Result<()> {
        let classifier: EncounterableResourcePathClassifier = Default::default();
        let mut erc = EncounterableResourceClass {
            flags: EncounterableResourceFlags::empty(),
            nature: None,
        };
        if classifier.classify(fs_path, &mut erc)
            && erc
                .flags
                .contains(EncounterableResourceFlags::CAPTURABLE_EXECUTABLE)
        {
            let ce = CapturableExecutable::from_executable_file_path(
                std::path::Path::new(fs_path),
                &erc,
            );
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
                CapturableExecutable::RequestedButNotExecutable(uri) => {
                    (uri.clone(), &unknown_nature, &false)
                }
            };
            info!("src: {}", src);
            info!("nature: {} (is batch SQL: {})", nature, is_batch_sql);
            let mut emitted = 0;

            if nature == "json" {
                info!("{:?}", ce.executed_result_as_json(stdin.clone()));
                emitted += 1;
            }

            if nature == "surveilr-SQL" {
                info!("{:?}", ce.executed_result_as_sql(stdin.clone()));
                emitted += 1;
            }

            if emitted == 0 {
                match ce.executed_result_as_text(stdin) {
                    Ok((stdout_text, _nature, _is_batch_sql)) => {
                        info!("{}", stdout_text)
                    }
                    Err(error_json) => {
                        error!("{}", serde_json::to_string_pretty(&error_json).unwrap())
                    }
                }
            }
        } else {
            error!("Unable to classify {} as a capturable executable.", fs_path)
        }

        Ok(())
    }

    fn task(
        &self,
        read_from_stdin: bool,
        task_cmds: &[String],
        _cwd: Option<&String>,
    ) -> anyhow::Result<()> {
        debug!("{:?}", task_cmds);

        let tasks = if read_from_stdin {
            std::io::stdin()
                .lines()
                .map(Result::ok)
                .map(|t| t.unwrap())
                .collect()
        } else {
            task_cmds.to_vec()
        };

        let (_, resources) = ResourcesCollection::from_tasks_lines(
            &tasks,
            &Default::default(),
            &None::<HashMap<_, _>>,
        );
        for ur in resources.uniform_resources() {
            match ur {
                Ok(resource) => match &resource {
                    UniformResource::CapturableExec(cer) => {
                        info!("URI: '{}', nature: {:?}", resource.uri(), resource.nature());
                        let stdin = shell::ShellStdIn::None;
                        match &cer.resource.nature {
                            Some(nature) => match nature.as_str() {
                                "json" | "text/json" | "application/json" => {
                                    match cer.executable.executed_result_as_json(stdin) {
                                        Ok((json_value, _nature, _is_sql_exec)) => {
                                            info!(
                                                "{}",
                                                serde_json::to_string_pretty(&json_value).unwrap()
                                            );
                                        }
                                        Err(err) => {
                                            error!("ERROR in JSON -- did you remember to have your command output JSON?\n{:?}", err);
                                        }
                                    }
                                }
                                _ => match cer.executable.executed_result_as_text(stdin) {
                                    Ok((stdout, _nature, _is_sql_exec)) => {
                                        info!("{stdout}");
                                    }
                                    Err(err) => {
                                        error!("ERROR in text\n{:?}", err);
                                    }
                                },
                            },
                            None => {
                                error!("Ideterminate nature");
                            }
                        }
                    }
                    _ => {
                        error!("Can only handle UniformResource::CapturableExec resources");
                    }
                },
                Err(e) => {
                    error!("Error processing a ingest_tasks resource: {}", e);
                }
            }
        }

        Ok(())
    }
}
