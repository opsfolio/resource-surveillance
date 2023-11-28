use std::collections::HashMap;

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;

use super::WalkerCommands;
use crate::resource::*;
use crate::rwalk::*;

// Implement methods for `AdminCommands`, ensure that whether the commands
// are called from CLI or natively within Rust, all the calls remain ergonomic.
impl WalkerCommands {
    pub fn execute(&self, cli: &super::Cli, _args: &super::WalkerArgs) -> anyhow::Result<()> {
        match self {
            WalkerCommands::Stats(ls_args) => self.stats(
                cli,
                &ResourceCollectionOptions {
                    physical_fs_root_paths: ls_args.root_fs_path.to_vec(),
                    acquire_content_regexs: ls_args.surveil_fs_content.to_vec(),
                    ignore_paths_regexs: ls_args.ignore_fs_entry.to_vec(),
                    capturable_executables_regexs: ls_args.capture_fs_exec.to_vec(),
                    captured_exec_sql_regexs: ls_args.captured_fs_exec_sql.to_vec(),
                    nature_bind: ls_args.nature_bind.clone().unwrap_or(HashMap::default()),
                },
            ),
        }
    }

    fn stats(&self, _cli: &super::Cli, options: &ResourceCollectionOptions) -> anyhow::Result<()> {
        let resources = ResourceCollection::new(options);

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                "Walked",
                &resources.walked.len().to_string(),
                "Rule(s)",
            ]);
        let column = table.column_mut(1).expect("Our table has two columns");
        column.set_cell_alignment(CellAlignment::Right);

        table.add_row(vec![
            Cell::new("Ignored"),
            Cell::new(resources.ignored().count().to_string()),
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
            Cell::new(resources.not_ignored().count().to_string()),
            Cell::new("All files not ignored"),
        ]);
        table.add_row(vec![
            "Inspectable",
            &resources
                .content_resources()
                .filter(|crs| !matches!(crs, ContentResourceSupplied::Ignored(_)))
                .count()
                .to_string(),
            "Files surveilr knows how to handle",
        ]);

        let uniform_resources: Vec<_> = resources
            .uniform_resources()
            .filter_map(Result::ok)
            .collect();
        table.add_row(vec![
            "Potential Uniform Resources",
            &resources.uniform_resources().count().to_string(),
        ]);
        table.add_row(vec![
            Cell::new("Ok").set_alignment(CellAlignment::Right),
            Cell::new(&uniform_resources.len().to_string()),
            Cell::new("Files surveilr can construct Uniform Resources for"),
        ]);
        table.add_row(vec![
            Cell::new("Err").set_alignment(CellAlignment::Right),
            Cell::new(
                &resources
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
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                "Uniform Resources",
                &uniform_resources.len().to_string(),
                "Rule(s)",
            ]);
        let column = table.column_mut(1).expect("Our table has two columns");
        column.set_cell_alignment(CellAlignment::Right);

        table.add_row(vec![
            Cell::new("Capturable Executables"),
            Cell::new(resources.capturable_executables().count().to_string()),
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
                uniform_resources
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
                resources
                    .content_resources()
                    .filter(|crs| match crs {
                        ContentResourceSupplied::Resource(cr) => cr.content_text_supplier.is_some(),
                        _ => false,
                    })
                    .count()
                    .to_string(),
            ),
            Cell::new(
                options
                    .acquire_content_regexs
                    .iter()
                    .map(|re| re.to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
        ]);

        let natures = uniform_resources
            .iter()
            .filter(|ur| !matches!(ur, UniformResource::Unknown(_, _)))
            .map(|ur| (ur.nature().clone().unwrap_or("UNKNOWN".to_string()), 1))
            .fold(HashMap::new(), |mut acc, (nature, count)| {
                *acc.entry(nature.clone()).or_insert(0) += count;
                acc
            });

        let mut sorted_natures: Vec<_> = natures.iter().collect();
        sorted_natures.sort_by_key(|&(k, _)| k);
        for (nature, count) in sorted_natures {
            table.add_row(vec![
                Cell::new(nature.to_string()).set_alignment(CellAlignment::Right),
                Cell::new(count.to_string()),
            ]);
        }

        println!("\n{table}");

        Ok(())
    }
}
