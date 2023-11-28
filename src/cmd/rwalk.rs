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
                &ResourceWalkerOptions {
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

    fn stats(&self, _cli: &super::Cli, options: &ResourceWalkerOptions) -> anyhow::Result<()> {
        let walker = ResourceWalker::new(options);

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["Statistic", "Value", "Rule(s)"]);
        let column = table.column_mut(1).expect("Our table has two columns");
        column.set_cell_alignment(CellAlignment::Right);

        table.add_row(vec!["Walked", &walker.all().count().to_string()]);
        table.add_row(vec![
            Cell::new("Ignored").set_alignment(CellAlignment::Right),
            Cell::new(walker.ignored().count().to_string()),
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
            Cell::new("Available").set_alignment(CellAlignment::Right),
            Cell::new(walker.not_ignored().count().to_string()),
        ]);
        table.add_row(vec![
            "Inspectable",
            &walker
                .content_resources()
                .filter(|crs| !matches!(crs, ContentResourceSupplied::Ignored(_)))
                .count()
                .to_string(),
        ]);
        table.add_row(vec![
            "Uniformable",
            &walker.uniform_resources().count().to_string(),
        ]);
        table.add_row(vec![
            Cell::new("Ok").set_alignment(CellAlignment::Right),
            Cell::new(
                walker
                    .uniform_resources()
                    .filter(|ur| ur.is_ok())
                    .count()
                    .to_string(),
            ),
        ]);
        table.add_row(vec![
            Cell::new("Err").set_alignment(CellAlignment::Right),
            Cell::new(
                walker
                    .uniform_resources()
                    .filter(|ur| !ur.is_ok())
                    .count()
                    .to_string(),
            ),
        ]);
        table.add_row(vec![
            Cell::new("Cap Execs").set_alignment(CellAlignment::Right),
            Cell::new(walker.capturable_executables().count().to_string()),
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
            Cell::new("Contenful").set_alignment(CellAlignment::Right),
            Cell::new(
                walker
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

        println!("{table}");

        let natures = walker
            .uniform_resources()
            .filter_map(|ur| ur.ok())
            .map(|ur| (ur.nature().clone().unwrap_or("UNKNOWN".to_string()), 1))
            .fold(HashMap::new(), |mut acc, (nature, count)| {
                *acc.entry(nature.clone()).or_insert(0) += count;
                acc
            });

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["UR Nature", "Count"]);
        let column = table.column_mut(1).expect("Our table has two columns");
        column.set_cell_alignment(CellAlignment::Right);

        let mut sorted_natures: Vec<_> = natures.iter().collect();
        sorted_natures.sort_by_key(|&(k, _)| k);
        for (nature, count) in sorted_natures {
            table.add_row(vec![
                Cell::new(nature.to_string()),
                Cell::new(count.to_string()),
            ]);
        }

        println!("\n{table}");

        Ok(())
    }
}
