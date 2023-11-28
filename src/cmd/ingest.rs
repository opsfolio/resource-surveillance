use rusqlite::{Connection, OpenFlags};

use crate::persist::*;

impl super::IngestArgs {
    pub fn execute(&self, cli: &super::Cli) -> anyhow::Result<()> {
        match crate::ingest::ingest(cli, self) {
            Ok(ingest_session_id) => {
                if self.stats || self.stats_json {
                    if let Ok(conn) = Connection::open_with_flags(
                        self.state_db_fs_path.clone(),
                        OpenFlags::SQLITE_OPEN_READ_ONLY,
                    ) {
                        if self.stats_json {
                            if let Ok(stats) =
                                ingest_session_stats_latest(&conn, ingest_session_id.clone())
                            {
                                print!("{}", serde_json::to_string_pretty(&stats).unwrap())
                            }
                        }

                        if self.stats {
                            let mut rows: Vec<Vec<String>> = Vec::new(); // Declare the rows as a vector of vectors of strings
                            ingest_session_stats(
                                &conn,
                                |_index,
                                 root_path,
                                 file_extension,
                                 file_count,
                                 with_content_count,
                                 with_frontmatter_count| {
                                    if self.root_fs_path.len() < 2 {
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
                                if self.root_fs_path.len() < 2 {
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
                            "Notebooks cells command requires a database: {}",
                            self.state_db_fs_path
                        );
                    }
                }
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}
