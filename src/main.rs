use std::path::PathBuf;

use clap::{Parser, Subcommand};
use regex::Regex;
use rusqlite::{Connection, OpenFlags};

#[macro_use]
extern crate lazy_static;

mod device;
lazy_static! {
    static ref DEVICE: device::Device = device::Device::new(None);
}

#[macro_use]
mod helpers;

mod format;
mod fsresource;
mod persist;
mod resource;

use format::*;
use fsresource::*;
use persist::*;
use resource::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Generate a Markdown file of all CLI commands and options
    #[arg(long)]
    help_markdown: bool,

    /// How to identify this device
    #[arg(long, num_args = 0..=1, default_value = DEVICE.name(), default_missing_value = "always")]
    device_name: Option<String>,

    /// TODO: Use a Deno *.ts or Nickel config file for defaults, allowing CLI args as overrides
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// TODO: Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Notebooks' cells emit utilities
    CatCells {
        /// target SQLite database
        #[arg(
            short,
            long,
            default_value = "./resource-surveillance.sqlite.db",
            default_missing_value = "always"
        )]
        db_fs_path: Option<String>,

        // search for these notebooks (include % for LIKE otherwise =)
        #[arg(short, long)]
        notebook: Vec<String>,

        // search for these cells (include % for LIKE otherwise =)
        #[arg(short, long)]
        cell: Vec<String>,
    },

    /// Notebooks maintenance utilities
    Notebooks {
        /// target SQLite database
        #[arg(
            short,
            long,
            default_value = "./resource-surveillance.sqlite.db",
            default_missing_value = "always"
        )]
        db_fs_path: Option<String>,
    },

    /// Walks the device file system
    FsWalk {
        /// one or more root paths to walk
        #[arg(short, long, default_value = ".", default_missing_value = "always")]
        root_path: Vec<String>,

        /// reg-exes to use to ignore files in root-path(s)
        #[arg(
            short,
            long,
            default_value = "/(\\.git|node_modules)/",
            default_missing_value = "always"
        )]
        ignore_entry: Vec<Regex>,

        /// reg-exes to use to compute digests for
        #[arg(long, default_value = ".*", default_missing_value = "always")]
        compute_digests: Vec<Regex>,

        /// reg-exes to use to load content for entry instead of just walking
        #[arg(
            long,
            default_value = "\\.(md|mdx|html|json)$",
            default_missing_value = "always"
        )]
        surveil_content: Vec<Regex>,

        /// target SQLite database
        #[arg(
            long,
            default_value = "./resource-surveillance.sqlite.db",
            default_missing_value = "always"
        )]
        surveil_db_fs_path: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    if cli.help_markdown {
        clap_markdown::print_help_markdown::<Cli>();
        return;
    }

    // --debug can be passed more than once to increase level
    match cli.debug {
        0 => {}
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    if cli.debug == 1 {
        // You can check the value provided by positional arguments, or option arguments
        if let Some(name) = cli.device_name.as_deref() {
            println!("Device: {name}");
        }

        if let Some(config_path) = cli.config.as_deref() {
            println!("config: {}", config_path.display());
        }
    }

    match &cli.command {
        Some(Commands::CatCells {
            db_fs_path,
            notebook: notebooks,
            cell: cells,
        }) => {
            if let Some(db_fs_path) = db_fs_path.as_deref() {
                if let Ok(conn) =
                    Connection::open_with_flags(db_fs_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
                {
                    match select_notebooks_and_cells(&conn, notebooks, cells) {
                        Ok(matched) => {
                            for row in matched {
                                let (notebook, kernel, cell, sql) = row;
                                println!("-- {notebook}::{cell} ({kernel})");
                                println!("{sql}");
                            }
                        }
                        Err(err) => println!("Notebooks cells command error: {}", err),
                    }
                } else {
                    println!(
                        "Notebooks cells command requires a database: {}",
                        db_fs_path
                    );
                }
            };
        }
        Some(Commands::Notebooks { db_fs_path }) => {
            if let Some(db_fs_path) = db_fs_path.as_deref() {
                if let Ok(conn) =
                    Connection::open_with_flags(db_fs_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
                {
                    let mut rows: Vec<Vec<String>> = Vec::new(); // Declare the rows as a vector of vectors of strings
                    notebook_cells(&conn, |_index, kernel, nb, cell, id| {
                        rows.push(vec![nb, kernel, cell, id]);
                        Ok(())
                    })
                    .unwrap();
                    println!(
                        "{}",
                        format_table(&["Notebook", "Kernel", "Cell", "ID"], &rows)
                    );

                    rows = Vec::new(); // Declare the rows as a vector of vectors of strings
                    notebook_cell_states(
                        &conn,
                        |_index,
                         _code_notebook_state_id,
                         notebook_name,
                         cell_name,
                         notebook_kernel_id,
                         from_state,
                         to_state,
                         transition_reason,
                         transitioned_at| {
                            rows.push(vec![
                                notebook_name,
                                notebook_kernel_id,
                                cell_name,
                                from_state,
                                to_state,
                                transition_reason,
                                transitioned_at,
                            ]);
                            Ok(())
                        },
                    )
                    .unwrap();
                    println!(
                        "{}",
                        format_table(
                            &["Notebook", "Kernel", "Cell", "From", "To", "Remarks", "When"],
                            &rows
                        )
                    );
                } else {
                    println!("Notebooks command requires a database: {}", db_fs_path);
                };
            }
        }
        Some(Commands::FsWalk {
            root_path: root_paths,
            ignore_entry: ignore_entries,
            surveil_content,
            surveil_db_fs_path,
            compute_digests,
        }) => {
            if let Some(db_fs_path) = surveil_db_fs_path.as_deref() {
                if cli.debug == 1 {
                    println!("Surveillance DB: {db_fs_path}");
                }

                if let Ok(mut conn) = Connection::open(db_fs_path) {
                    // putting everything inside a transaction improves performance significantly
                    let tx = conn.transaction().unwrap();

                    match execute_migrations(&tx) {
                        Ok(_) => {}
                        Err(err) => {
                            println!("execute_migrations Error {}", err);
                        }
                    };

                    match upserted_device(&tx,&DEVICE) {
                        Ok((device_id, device_name)) => {
                            if cli.debug == 1 { println!("Device: {device_name} ({device_id})"); }

                            // TODO: figure out why so many .clone() are necessary instead of pointers
                            let walk_session_id = ulid::Ulid::new().to_string();
                            if cli.debug == 1 { println!("Walk Session: {walk_session_id}"); }
                            match tx.execute(r"
                                INSERT INTO fs_content_walk_session (fs_content_walk_session_id, device_id, ignore_paths_regex, blobs_regex, digests_regex, walk_started_at) 
                                                                VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)", [
                                                                walk_session_id.clone(), device_id, 
                                                                ignore_entries.iter().map(|r| r.as_str()).collect::<Vec<&str>>().join(", "), 
                                                                compute_digests.iter().map(|r| r.as_str()).collect::<Vec<&str>>().join(", "), 
                                                                surveil_content.iter().map(|r| r.as_str()).collect::<Vec<&str>>().join(", ")]) {
                                Ok(_) => {
                                    // TODO: don't unwrap, handle errors properly
                                    let mut fscwp_stmt = tx.prepare("INSERT INTO fs_content_walk_path (fs_content_walk_path_id, walk_session_id, root_path) VALUES (?, ?, ?)").unwrap();
                                    let mut fsc_stmt = tx.prepare("INSERT INTO fs_content (fs_content_id, walk_session_id, walk_path_id, file_path, content_digest, file_bytes, file_mtime) VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT (content_digest, file_path, file_bytes, file_mtime) DO NOTHING").unwrap();
                                    for root_path in root_paths {
                                        match std::fs::canonicalize(std::path::Path::new(root_path)) {
                                            Ok(canonical_path_buf) => {
                                                let canonical_path = canonical_path_buf.into_os_string().into_string().unwrap();
                                                let walk_path_id = ulid::Ulid::new().to_string();
                                                if cli.debug == 1 { println!("  Walk Session Path: {root_path} ({walk_path_id})"); }                                                    
                                                match fscwp_stmt.execute([walk_path_id.clone(), walk_session_id.clone(), canonical_path.clone()]) {
                                                    Ok(_) => {
                                                        let rp: Vec<String> = vec![canonical_path];
                                                        let walker = FileSysResourcesWalker::new(&rp, ignore_entries, surveil_content);
                                                        match walker {
                                                            Ok(walker) => {
                                                                for resource_result in walker.walk_resources_iter() {
                                                                    match resource_result {
                                                                        Ok(resource) => {
                                                                            let fs_content_id = ulid::Ulid::new().to_string();
                                                                            match resource {
                                                                                UniformResource::Html(_html) => {
                                                                                    // println!("HTML: {:?} {:?}", html.resource.uri, html.resource.nature)
                                                                                }
                                                                                UniformResource::Json(_json) => {
                                                                                    // println!("JSON: {:?} {:?}", json.resource.uri, json.resource.nature)
                                                                                }
                                                                                UniformResource::Image(_img) => {
                                                                                    // println!("Image: {:?} {:?}", img.resource.uri, img.resource.nature)
                                                                                }
                                                                                UniformResource::Markdown(_md) => {
                                                                                    // println!("Markdown: {:?} {:?}", md.resource.uri, md.resource.nature)
                                                                                }
                                                                                UniformResource::Unknown(unknown) => {
                                                                                    let _execute = fsc_stmt.execute([
                                                                                        fs_content_id, walk_session_id.clone(), walk_path_id.clone(), 
                                                                                        unknown.uri, String::from('-'), unknown.size.unwrap().to_string(), 
                                                                                        unknown.last_modified_at.unwrap().to_string()]);
                                                                                    // println!("Unknown: {:?} {:?}", unknown.uri, unknown.nature)
                                                                                }
                                                                            }                                                                            },
                                                                        Err(e) => {
                                                                            // Handle errors here
                                                                            eprintln!("Error processing a resource: {}", e);
                                                                        },
                                                                    }
                                                                }                                                                                                                                    
                                                            }
                                                            Err(err) => { print!("Error preparing walker: {err}");}
                                                        }     
                                                    }
                                                    Err(err) => {
                                                        println!("fs_content_walk_path Error {}", err);
                                                    }
                                            };
                                        }
                                        Err(err) => { print!("Error canonicalizing path {root_path}: {err}");}
                                        }
                                    }

                                    let _ = tx.execute("UPDATE fs_content_walk_session SET walk_finished_at = CURRENT_TIMESTAMP WHERE fs_content_walk_session_id = ?", [walk_session_id.clone()]);
                                }
                                Err(err) => {
                                    println!("fs_content_walk_session Error {}", err);
                                }
                            };
                        }
                        Err(err) => {
                            println!("Unable to upsert device: {}", err);
                        }
                    };

                    // putting everything inside a transaction improves performance significantly
                    let _ = tx.commit();
                } else {
                    println!("RusqliteContext Could not open or create: {}", db_fs_path);
                };
            }
        }
        None => {}
    }
}
