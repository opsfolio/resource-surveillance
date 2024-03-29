use std::collections::HashMap;

use autometrics::autometrics;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::*;
use serde_rusqlite::rusqlite;
use tracing::info;

use resource::*;
use resource_serde::cmd::{IngestArgs, IngestCommands, IngestFilesArgs, IngestTasksArgs};
use resource_serde::{ingest, persist::*};

// Implement methods for `AdminCommands`, ensure that whether the commands
// are called from CLI or natively within Rust, all the calls remain ergonomic.
#[derive(Debug, Default)]
pub struct Ingest {}

impl Ingest {
    #[autometrics]
    pub async fn execute(&self, cli: &super::Cli, args: &IngestArgs) -> anyhow::Result<()> {
        match &args.command {
            IngestCommands::Files(ifa) => {
                if ifa.dry_run {
                    self.files_dry_run(cli, &ifa.root_fs_path, ifa)
                } else {
                    self.files(cli, ifa)
                }
            }
            IngestCommands::Tasks(ifa) => self.tasks(cli, ifa),
            IngestCommands::Imap(ima) => ingest::ingest_imap(ima).await,
        }
    }

    fn files(&self, cli: &super::Cli, args: &IngestFilesArgs) -> anyhow::Result<()> {
        match ingest::ingest_files(cli.debug, args) {
            Ok(ingest_session_id) => {
                if args.stats || args.stats_json {
                    // only export the path if there's more than one
                    let sql = if args.root_fs_path.len() > 1 || args.stats_json {
                        r"SELECT ingest_session_root_fs_path as 'Path',
                                 file_extension as 'Extn',
                                 total_file_count AS 'Count',
                                 file_count_with_content AS 'Content',
                                 file_count_with_frontmatter AS 'Frontmatter'
                            FROM ur_ingest_session_files_stats
                           WHERE ingest_session_id = ?"
                    } else {
                        r"SELECT file_extension as 'Extn',
                                 total_file_count AS 'Count',
                                 file_count_with_content AS 'Content',
                                 file_count_with_frontmatter AS 'Frontmatter'
                            FROM ur_ingest_session_files_stats
                           WHERE ingest_session_id = ?"
                    };

                    let dbc = DbConn::open(&args.state_db_fs_path, cli.debug)?;
                    if args.stats_json {
                        let value = dbc.query_result_as_json_value(
                            sql,
                            rusqlite::params![ingest_session_id],
                        )?;
                        info!("{}", serde_json::to_string_pretty(&value)?);
                    } else {
                        let table = dbc.query_result_as_formatted_table(
                            sql,
                            rusqlite::params![ingest_session_id],
                        )?;
                        info!(
                            "\n==> `ur_ingest_session_files_stats` for session ID '{}':\n{}",
                            ingest_session_id, table
                        )
                    }
                }
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn tasks(&self, cli: &super::Cli, args: &IngestTasksArgs) -> anyhow::Result<()> {
        match ingest::ingest_tasks(cli.debug, args) {
            Ok(ingest_session_id) => {
                if args.stats || args.stats_json {
                    let sql = r#"
                        SELECT ur_status as 'Status', 
                            nature as 'Nature', 
                            total_file_count as 'Count', 
                            file_count_with_content as 'Content', 
                            file_count_with_frontmatter as 'Frontmatter'
                         FROM ur_ingest_session_tasks_stats_latest
                        WHERE ingest_session_id = ?"#;

                    let dbc = DbConn::open(&args.state_db_fs_path, cli.debug)?;
                    if args.stats_json {
                        let value = dbc.query_result_as_json_value(
                            sql,
                            rusqlite::params![ingest_session_id],
                        )?;
                        info!("{}", serde_json::to_string_pretty(&value)?);
                    } else {
                        let table = dbc.query_result_as_formatted_table(
                            sql,
                            rusqlite::params![ingest_session_id],
                        )?;
                        info!(
                            "\n==> `ur_ingest_session_tasks_stats` for session ID '{}':\n{}",
                            ingest_session_id, table
                        )
                    }
                }
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn files_dry_run(
        &self,
        _cli: &super::Cli,
        root_fs_path: &[String],
        _args: &IngestFilesArgs,
    ) -> anyhow::Result<()> {
        let classifier = EncounterableResourcePathClassifier::default();
        let wd_resources =
            ResourcesCollection::from_walk_dir(root_fs_path, &classifier, &None::<HashMap<_, _>>);
        let si_resources =
            ResourcesCollection::from_smart_ignore(root_fs_path, &classifier, None, false);
        let vfs_pfs_resources =
            ResourcesCollection::from_vfs_physical_fs(root_fs_path, &classifier, None);

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL_CONDENSED)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["", "WalkDir", "SmartIgnore", "VFS_PFS", "Rule(s)"]);
        table
            .column_mut(1)
            .expect("Our table has two columns")
            .set_cell_alignment(CellAlignment::Right);
        table
            .column_mut(2)
            .expect("Our table has three columns")
            .set_cell_alignment(CellAlignment::Right);
        table
            .column_mut(3)
            .expect("Our table has three columns")
            .set_cell_alignment(CellAlignment::Right);

        table.add_row(vec![
            Cell::new("Encounterable Resources"),
            Cell::new(wd_resources.encounterable.len().to_string()),
            Cell::new(si_resources.encounterable.len().to_string()),
            Cell::new(vfs_pfs_resources.encounterable.len().to_string()),
            Cell::new("Files surveilr could potentially handle"),
        ]);
        table.add_row(vec![
            Cell::new("Ignored via filename Regex"),
            Cell::new(wd_resources.ignored().count().to_string()),
            Cell::new(si_resources.ignored().count().to_string()),
            Cell::new(vfs_pfs_resources.ignored().count().to_string()),
        ]);
        table.add_row(vec![
            Cell::new("Available"),
            Cell::new(wd_resources.not_ignored().count().to_string()),
            Cell::new(si_resources.not_ignored().count().to_string()),
            Cell::new(vfs_pfs_resources.not_ignored().count().to_string()),
            Cell::new("All files not ignored via filename Regex"),
        ]);
        table.add_row(vec![
            "Encountered Resources",
            &wd_resources
                .encountered()
                .filter(|crs| !matches!(crs, EncounteredResource::Ignored(_, _)))
                .count()
                .to_string(),
            &si_resources
                .encountered()
                .filter(|crs| !matches!(crs, EncounteredResource::Ignored(_, _)))
                .count()
                .to_string(),
            &vfs_pfs_resources
                .encountered()
                .filter(|crs| !matches!(crs, EncounteredResource::Ignored(_, _)))
                .count()
                .to_string(),
            "Files surveilr knows how to handle",
        ]);

        let wd_uniform_resources: Vec<_> = wd_resources
            .uniform_resources()
            .filter_map(Result::ok)
            .collect();
        let si_uniform_resources: Vec<_> = si_resources
            .uniform_resources()
            .filter_map(Result::ok)
            .collect();
        let vfs_pfs_uniform_resources: Vec<_> = vfs_pfs_resources
            .uniform_resources()
            .filter_map(Result::ok)
            .collect();

        table.add_row(vec![
            "Potential Uniform Resources",
            &wd_resources.uniform_resources().count().to_string(),
            &si_resources.uniform_resources().count().to_string(),
            &vfs_pfs_resources.uniform_resources().count().to_string(),
        ]);
        table.add_row(vec![
            Cell::new("Ok").set_alignment(CellAlignment::Right),
            Cell::new(wd_uniform_resources.len().to_string()),
            Cell::new(si_uniform_resources.len().to_string()),
            Cell::new(vfs_pfs_uniform_resources.len().to_string()),
            Cell::new("Files surveilr can construct Uniform Resources for"),
        ]);
        table.add_row(vec![
            Cell::new("Err").set_alignment(CellAlignment::Right),
            Cell::new(
                wd_resources
                    .uniform_resources()
                    .filter(|ur| ur.is_err())
                    .count()
                    .to_string(),
            ),
            Cell::new(
                si_resources
                    .uniform_resources()
                    .filter(|ur| ur.is_err())
                    .count()
                    .to_string(),
            ),
            Cell::new(
                vfs_pfs_resources
                    .uniform_resources()
                    .filter(|ur| ur.is_err())
                    .count()
                    .to_string(),
            ),
            Cell::new("Files surveilr cannot construct Uniform Resources for"),
        ]);

        println!("{table}");

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL_CONDENSED)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["", "WalkDir", "SmartIgnore", "VFS_PFS", "Rule(s)"]);
        table
            .column_mut(1)
            .expect("Our table has two columns")
            .set_cell_alignment(CellAlignment::Right);
        table
            .column_mut(2)
            .expect("Our table has three columns")
            .set_cell_alignment(CellAlignment::Right);
        table
            .column_mut(3)
            .expect("Our table has three columns")
            .set_cell_alignment(CellAlignment::Right);

        table.add_row(vec![
            Cell::new("Uniform Resources"),
            Cell::new(wd_uniform_resources.len().to_string()),
            Cell::new(si_uniform_resources.len().to_string()),
            Cell::new(vfs_pfs_uniform_resources.len().to_string()),
        ]);

        table.add_row(vec![
            Cell::new("Capturable Executables"),
            Cell::new(wd_resources.capturable_executables().count().to_string()),
            Cell::new(si_resources.capturable_executables().count().to_string()),
            Cell::new(
                vfs_pfs_resources
                    .capturable_executables()
                    .count()
                    .to_string(),
            ),
        ]);

        table.add_row(vec![
            Cell::new("Not Loadable"),
            Cell::new(
                wd_uniform_resources
                    .iter()
                    .filter(|ur| matches!(ur, UniformResource::Unknown(_, _)))
                    .count()
                    .to_string(),
            ),
            Cell::new(
                si_uniform_resources
                    .iter()
                    .filter(|ur| matches!(ur, UniformResource::Unknown(_, _)))
                    .count()
                    .to_string(),
            ),
            Cell::new(
                vfs_pfs_uniform_resources
                    .iter()
                    .filter(|ur| matches!(ur, UniformResource::Unknown(_, _)))
                    .count()
                    .to_string(),
            ),
            Cell::new("unknown `nature`"),
        ]);

        table.add_row(vec![
            Cell::new("Content text suppliers"),
            Cell::new(
                wd_resources
                    .encountered()
                    .filter(|crs| match crs {
                        EncounteredResource::Resource(cr, _) => cr.content_text_supplier.is_some(),
                        _ => false,
                    })
                    .count()
                    .to_string(),
            ),
            Cell::new(
                si_resources
                    .encountered()
                    .filter(|crs| match crs {
                        EncounteredResource::Resource(cr, _) => cr.content_text_supplier.is_some(),
                        _ => false,
                    })
                    .count()
                    .to_string(),
            ),
            Cell::new(
                vfs_pfs_resources
                    .encountered()
                    .filter(|crs| match crs {
                        EncounteredResource::Resource(cr, _) => cr.content_text_supplier.is_some(),
                        _ => false,
                    })
                    .count()
                    .to_string(),
            ),
        ]);

        let wd_natures = wd_uniform_resources
            .iter()
            .filter(|ur| !matches!(ur, UniformResource::Unknown(_, _)))
            .map(|ur| (ur.nature().clone().unwrap_or("UNKNOWN".to_string()), 1))
            .fold(HashMap::new(), |mut acc, (nature, count)| {
                *acc.entry(nature.clone()).or_insert(0) += count;
                acc
            });
        let si_natures = si_uniform_resources
            .iter()
            .filter(|ur| !matches!(ur, UniformResource::Unknown(_, _)))
            .map(|ur| (ur.nature().clone().unwrap_or("UNKNOWN".to_string()), 1))
            .fold(HashMap::new(), |mut acc, (nature, count)| {
                *acc.entry(nature.clone()).or_insert(0) += count;
                acc
            });
        let vps_pfs_natures = vfs_pfs_uniform_resources
            .iter()
            .filter(|ur| !matches!(ur, UniformResource::Unknown(_, _)))
            .map(|ur| (ur.nature().clone().unwrap_or("UNKNOWN".to_string()), 1))
            .fold(HashMap::new(), |mut acc, (nature, count)| {
                *acc.entry(nature.clone()).or_insert(0) += count;
                acc
            });

        let mut sorted_natures: Vec<_> = wd_natures
            .keys()
            .chain(si_natures.keys())
            .chain(vps_pfs_natures.keys())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        sorted_natures.sort();

        for nature in &sorted_natures {
            table.add_row(vec![
                Cell::new(nature).set_alignment(CellAlignment::Right),
                Cell::new(
                    wd_natures
                        .get(nature)
                        .map(|v| v.to_string())
                        .unwrap_or_else(String::new),
                )
                .set_alignment(CellAlignment::Right),
                Cell::new(
                    si_natures
                        .get(nature)
                        .map(|v| v.to_string())
                        .unwrap_or_else(String::new),
                )
                .set_alignment(CellAlignment::Right),
                Cell::new(
                    vps_pfs_natures
                        .get(nature)
                        .map(|v| v.to_string())
                        .unwrap_or_else(String::new),
                )
                .set_alignment(CellAlignment::Right),
            ]);
        }

        println!("\n{table}");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use resource_serde::cmd::{IngestArgs, IngestCommands, IngestFilesArgs};

    use crate::{ingest::Ingest, Cli};

    fn build_cli(subcmd: &str, root_fs_path: &str, dry_run: bool) -> Cli {
        let mut fixtures_dir = std::env::current_dir().expect("Failed to get current directory");
        fixtures_dir.push("support/test-fixtures");
        let mut args = vec![
            "surveilr",
            "ingest",
            subcmd,
            "-d",
            "functional-test-state.sqlite.db",
            "-r",
            root_fs_path,
        ];

        if dry_run {
            args.push("--dry-run")
        };

        Cli::parse_from(args)
    }

    #[tokio::test]
    async fn test_dry_run() {
        let mut fixtures_dir = std::env::current_dir().expect("Failed to get current directory");
        fixtures_dir.push("../../support/test-fixtures");

        let ingest_file_args = IngestFilesArgs {
            dry_run: true,
            behavior: None,
            root_fs_path: vec![fixtures_dir.to_str().unwrap().to_string()],
            state_db_fs_path: "functional-test-state.sqlite.db".to_string(),
            state_db_init_sql: vec![],
            include_state_db_in_ingestion: false,
            stats: false,
            stats_json: false,
            save_behavior: None,
        };

        let cli = build_cli(
            "files",
            ingest_file_args.root_fs_path.first().unwrap(),
            ingest_file_args.dry_run,
        );
        let ingest_cmd = IngestCommands::Files(ingest_file_args);
        let ingest = Ingest::default();
        let res = ingest.execute(
            &cli,
            &IngestArgs {
                command: ingest_cmd.clone(),
            },
        ).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_file_ingestion() {
        let mut fixtures_dir = std::env::current_dir().expect("Failed to get current directory");
        fixtures_dir.push("../../support/test-fixtures");

        let ingest_file_args = IngestFilesArgs {
            dry_run: false,
            behavior: None,
            root_fs_path: vec![fixtures_dir.to_str().unwrap().to_string()],
            state_db_fs_path: "functional-test-state.sqlite.db".to_string(),
            state_db_init_sql: vec![],
            include_state_db_in_ingestion: false,
            stats: false,
            stats_json: false,
            save_behavior: None,
        };

        let cli = build_cli(
            "files",
            ingest_file_args.root_fs_path.first().unwrap(),
            ingest_file_args.dry_run,
        );
        let ingest_cmd = IngestCommands::Files(ingest_file_args);
        let ingest = Ingest::default();
        let res = ingest.execute(
            &cli,
            &IngestArgs {
                command: ingest_cmd.clone(),
            },
        ).await;
        assert!(res.is_ok());
    }
}
