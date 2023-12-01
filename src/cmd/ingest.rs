use std::collections::HashMap;

use rusqlite::{Connection, OpenFlags};

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::*;

use super::IngestCommands;
use crate::persist::*;
use crate::resource::*;

// Implement methods for `AdminCommands`, ensure that whether the commands
// are called from CLI or natively within Rust, all the calls remain ergonomic.
impl IngestCommands {
    pub fn execute(&self, cli: &super::Cli, _args: &super::IngestArgs) -> anyhow::Result<()> {
        match self {
            IngestCommands::Files(ifa) => {
                if ifa.dry_run {
                    self.files_dry_run(
                        cli,
                        &ifa.root_fs_path,
                        &ResourcesCollectionOptions {
                            ingest_content_regexs: ifa.surveil_fs_content.to_vec(),
                            ignore_paths_regexs: ifa.ignore_fs_entry.to_vec(),
                            capturable_executables_regexs: ifa.capture_fs_exec.to_vec(),
                            captured_exec_sql_regexs: ifa.captured_fs_exec_sql.to_vec(),
                            nature_bind: ifa.nature_bind.clone().unwrap_or(HashMap::default()),
                        },
                    )
                } else {
                    self.files(cli, ifa)
                }
            }
            IngestCommands::Tasks(ifa) => self.tasks(cli, ifa),
        }
    }

    fn session_stats(
        &self,
        state_db_fs_path: &str,
        root_fs_path: &[String],
        ingest_session_id: String,
        stats: bool,
        stats_json: bool,
    ) {
        if !stats && !stats_json {
            return;
        }

        if let Ok(conn) =
            Connection::open_with_flags(state_db_fs_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        {
            if stats_json {
                if let Ok(stats) = ingest_session_stats_latest(&conn, ingest_session_id.clone()) {
                    print!("{}", serde_json::to_string_pretty(&stats).unwrap())
                }
            }

            if stats {
                let mut rows: Vec<Vec<String>> = Vec::new(); // Declare the rows as a vector of vectors of strings
                ingest_session_stats(
                    &conn,
                    |_index,
                     root_path,
                     file_extension,
                     file_count,
                     with_content_count,
                     with_frontmatter_count| {
                        if root_fs_path.len() < 2 {
                            rows.push(vec![
                                file_extension,
                                file_count.to_string(),
                                with_content_count.to_string(),
                                with_frontmatter_count.to_string(),
                            ]);
                        } else {
                            rows.push(vec![
                                root_path,
                                file_extension,
                                file_count.to_string(),
                                with_content_count.to_string(),
                                with_frontmatter_count.to_string(),
                            ]);
                        }
                        Ok(())
                    },
                    ingest_session_id,
                )
                .unwrap();
                println!(
                    "{}",
                    if root_fs_path.len() < 2 {
                        crate::format::as_ascii_table(
                            &["Extn", "Count", "Content", "Frontmatter"],
                            &rows,
                        )
                    } else {
                        crate::format::as_ascii_table(
                            &["Path", "Extn", "Count", "Content", "Frontmatter"],
                            &rows,
                        )
                    }
                );
            }
        } else {
            println!(
                "Ingest files command requires a database: {}",
                state_db_fs_path
            );
        }
    }

    fn files(&self, cli: &super::Cli, args: &super::IngestFilesArgs) -> anyhow::Result<()> {
        match crate::ingest::ingest_files(cli, args) {
            Ok(ingest_session_id) => {
                self.session_stats(
                    &args.state_db_fs_path,
                    &args.root_fs_path,
                    ingest_session_id,
                    args.stats,
                    args.stats_json,
                );
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn tasks(&self, cli: &super::Cli, args: &super::IngestTasksArgs) -> anyhow::Result<()> {
        match crate::ingest::ingest_tasks(cli, args) {
            Ok(ingest_session_id) => {
                self.session_stats(
                    &args.state_db_fs_path,
                    &["cli-tasks".to_string()],
                    ingest_session_id,
                    true,
                    false,
                );
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn files_dry_run(
        &self,
        _cli: &super::Cli,
        root_fs_path: &[String],
        options: &ResourcesCollectionOptions,
    ) -> anyhow::Result<()> {
        let wd_resources = ResourcesCollection::from_walk_dir(root_fs_path, options);
        let si_resources = ResourcesCollection::from_smart_ignore(root_fs_path, options, false);
        let vfs_pfs_resources = ResourcesCollection::from_vfs_physical_fs(root_fs_path, options);

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
            Cell::new(
                options
                    .ignore_paths_regexs
                    .iter()
                    .map(|re| re.to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
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
                .filter(|crs| !matches!(crs, EncounteredResource::Ignored(_)))
                .count()
                .to_string(),
            &si_resources
                .encountered()
                .filter(|crs| !matches!(crs, EncounteredResource::Ignored(_)))
                .count()
                .to_string(),
            &vfs_pfs_resources
                .encountered()
                .filter(|crs| !matches!(crs, EncounteredResource::Ignored(_)))
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
            Cell::new(
                options
                    .capturable_executables_regexs
                    .clone()
                    .into_iter()
                    .chain(options.captured_exec_sql_regexs.clone())
                    .map(|re| re.to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
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
                        EncounteredResource::Resource(cr) => cr.content_text_supplier.is_some(),
                        _ => false,
                    })
                    .count()
                    .to_string(),
            ),
            Cell::new(
                si_resources
                    .encountered()
                    .filter(|crs| match crs {
                        EncounteredResource::Resource(cr) => cr.content_text_supplier.is_some(),
                        _ => false,
                    })
                    .count()
                    .to_string(),
            ),
            Cell::new(
                vfs_pfs_resources
                    .encountered()
                    .filter(|crs| match crs {
                        EncounteredResource::Resource(cr) => cr.content_text_supplier.is_some(),
                        _ => false,
                    })
                    .count()
                    .to_string(),
            ),
            Cell::new(
                options
                    .ingest_content_regexs
                    .iter()
                    .map(|re| re.to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
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
