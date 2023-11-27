use std::collections::HashMap;

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

        println!("        All: {}", walker.all().count());
        println!(
            "    Ignored: {} ('{}')",
            walker.ignored().count(),
            options
                .ignore_paths_regexs
                .iter()
                .map(|re| re.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!("  Available: {}", walker.not_ignored().count(),);
        println!(
            "Inspectable: {}",
            walker
                .content_resources()
                .filter(|crs| !matches!(crs, ContentResourceSupplied::Ignored(_)))
                .count()
        );

        println!("-----");
        println!("Uniformable: {}", walker.uniform_resources().count(),);
        println!(
            "         Ok: {}",
            walker.uniform_resources().filter(|ur| ur.is_ok()).count(),
        );
        println!(
            "        Err: {}",
            walker.uniform_resources().filter(|ur| !ur.is_ok()).count(),
        );
        println!(
            "  Cap Execs: {} ('{}' '{}')",
            walker.capturable_executables().count(),
            options
                .capturable_executables_regexs
                .iter()
                .map(|re| re.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            options
                .captured_exec_sql_regexs
                .iter()
                .map(|re| re.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!(
            "  Contenful: {} ('{}')",
            walker
                .content_resources()
                .filter(|crs| match crs {
                    ContentResourceSupplied::Resource(cr) => cr.content_text_supplier.is_some(),
                    _ => false,
                })
                .count(),
            options
                .acquire_content_regexs
                .iter()
                .map(|re| re.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let natures = walker
            .uniform_resources()
            .filter_map(|ur| ur.ok())
            .map(|ur| (ur.nature().clone().unwrap_or("UNKNOWN".to_string()), 1))
            .fold(HashMap::new(), |mut acc, (nature, count)| {
                *acc.entry(nature.clone()).or_insert(0) += count;
                acc
            });

        println!("-----");
        natures
            .iter()
            .for_each(|(nature, count)| println!("{}: {}", nature, count));

        Ok(())
    }
}
