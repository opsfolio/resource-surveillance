use std::path::PathBuf;

use clap::{Parser, Subcommand};
use regex::Regex;
use rusqlite::{params, Connection, OpenFlags};

#[macro_use]
extern crate lazy_static;

mod device;
lazy_static! {
    static ref DEVICE: device::Device = device::Device::new(None);
}

#[macro_use]
mod helpers;

mod format;
mod frontmatter;
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

                // TODO: put all errors into an error log table inside the database
                //       associated with each session (e.g. ur_walk_session_telemetry)
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

                            let walk_session_id = ulid::Ulid::new().to_string();
                            if cli.debug == 1 { println!("Walk Session: {walk_session_id}"); }
                            match tx.execute(r"
                                INSERT INTO ur_walk_session (ur_walk_session_id, device_id, ignore_paths_regex, blobs_regex, digests_regex, walk_started_at) 
                                                     VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)", params![
                                    walk_session_id, device_id, 
                                    ignore_entries.iter().map(|r| r.as_str()).collect::<Vec<&str>>().join(", "), 
                                    compute_digests.iter().map(|r| r.as_str()).collect::<Vec<&str>>().join(", "), 
                                    surveil_content.iter().map(|r| r.as_str()).collect::<Vec<&str>>().join(", ")]) {
                                Ok(_) => {
                                    // TODO: don't unwrap, handle errors properly
                                    let mut ur_wsp_stmt = tx.prepare("INSERT INTO ur_walk_session_path (ur_walk_session_path_id, walk_session_id, root_path) VALUES (?, ?, ?)").unwrap();
                                    let mut ur_no_content_stmt = tx.prepare("INSERT INTO uniform_resource (uniform_resource_id, device_id, walk_session_id, walk_path_id, uri, nature, content_digest, size_bytes, last_modified_at) VALUES (?, ?, ?, ?, ?, ?, '-', ?, ?) ON CONFLICT (device_id, content_digest, uri, size_bytes, last_modified_at) DO NOTHING").unwrap();
                                    let mut ur_with_content_stmt = tx.prepare("INSERT INTO uniform_resource (uniform_resource_id, device_id, walk_session_id, walk_path_id, uri, nature, content, content_digest, size_bytes, last_modified_at, content_fm_body_attrs, frontmatter) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT (device_id, content_digest, uri, size_bytes, last_modified_at) DO NOTHING").unwrap();
                                    let mut ur_fs_entry_stmt = tx.prepare("INSERT INTO ur_walk_session_path_fs_entry (ur_walk_session_path_fs_entry_id, walk_session_id, walk_path_id, uniform_resource_id, file_path_abs, file_path_rel_parent, file_path_rel, file_basename, file_extn) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)").unwrap();
                                    for root_path in root_paths {
                                        match std::fs::canonicalize(std::path::Path::new(root_path)) {
                                            Ok(canonical_path_buf) => {
                                                let canonical_path = canonical_path_buf.into_os_string().into_string().unwrap();
                                                let walk_path_id = ulid::Ulid::new().to_string();
                                                if cli.debug == 1 { println!("  Walk Session Path: {root_path} ({walk_path_id})"); }                                                    
                                                match ur_wsp_stmt.execute(params![walk_path_id, walk_session_id, canonical_path]) {
                                                    Ok(_) => {
                                                        // TODO: why is this clone required?
                                                        let rp: Vec<String> = vec![canonical_path.clone()];
                                                        let walker = FileSysResourcesWalker::new(&rp, ignore_entries, surveil_content);
                                                        match walker {
                                                            Ok(walker) => {
                                                                for resource_result in walker.walk_resources_iter() {
                                                                    match resource_result {
                                                                        Ok(resource) => {
                                                                            let uniform_resource_id = ulid::Ulid::new().to_string();
                                                                            let uri: String;
                                                                            match resource {
                                                                                UniformResource::Html(html) => {
                                                                                    uri = html.resource.uri.to_string();
                                                                                    // println!("HTML: {:?} {:?}", html.resource.uri, html.resource.nature)
                                                                                }
                                                                                UniformResource::Json(json) => {
                                                                                    uri = json.resource.uri.to_string();
                                                                                    let content_supplier = json.resource.content_text_supplier.unwrap()().unwrap();
                                                                                    let execute = ur_with_content_stmt.execute(params![
                                                                                        uniform_resource_id, device_id, walk_session_id, walk_path_id, 
                                                                                        json.resource.uri, json.resource.nature, 
                                                                                        content_supplier.content_text(),
                                                                                        content_supplier.content_digest_hash(),
                                                                                        json.resource.size, 
                                                                                        json.resource.last_modified_at.unwrap().to_string(),
                                                                                        &None::<String>, &None::<String>]);
                                                                                    if execute.is_err() { eprintln!("Error inserting UniformResource::Json for {}: {:?}", &uri, execute.err()); }
                                                                                }
                                                                                UniformResource::Image(img) => {
                                                                                    uri = img.resource.uri.to_string();
                                                                                    println!("TODO UniformResource::Image: {:?} {:?}", img.resource.uri, img.resource.nature)
                                                                                }
                                                                                UniformResource::Markdown(md) => {
                                                                                    uri = md.resource.uri.to_string();
                                                                                    let content_supplier = md.resource.content_text_supplier.unwrap()().unwrap();
                                                                                    let mut fm_attrs = None::<String>;
                                                                                    let mut fm_json: Option<String> = None::<String>;
                                                                                    let (_, fm_raw, fm_json_value, fm_body) = content_supplier.frontmatter();
                                                                                    if fm_json_value.is_ok()  {
                                                                                        fm_json = Some(serde_json::to_string_pretty(&fm_json_value.ok()).unwrap());
                                                                                        let fm_attrs_value = serde_json::json!({
                                                                                            "frontMatter": fm_raw.unwrap(),
                                                                                            "body": fm_body, 
                                                                                            "attrs": fm_json.clone().unwrap()
                                                                                        });
                                                                                        fm_attrs = Some(serde_json::to_string_pretty(&fm_attrs_value).unwrap());
                                                                                    }
                                                                                    let execute = ur_with_content_stmt.execute(params![
                                                                                        uniform_resource_id, device_id, walk_session_id, walk_path_id, 
                                                                                        md.resource.uri, md.resource.nature, 
                                                                                        content_supplier.content_text(),
                                                                                        content_supplier.content_digest_hash(),
                                                                                        md.resource.size, 
                                                                                        md.resource.last_modified_at.unwrap().to_string(),
                                                                                        fm_attrs, fm_json]);
                                                                                    if execute.is_err() { eprintln!("Error inserting UniformResource::Markdown for {}: {:?}", &uri, execute.err()); }
                                                                                }
                                                                                UniformResource::Unknown(unknown) => {
                                                                                    uri = unknown.uri.to_string();
                                                                                    let execute = ur_no_content_stmt.execute(params![
                                                                                        uniform_resource_id, device_id, walk_session_id, walk_path_id, 
                                                                                        unknown.uri, unknown.nature, unknown.size, 
                                                                                        unknown.last_modified_at.unwrap().to_string()]);
                                                                                    if execute.is_err() { eprintln!("Error inserting UniformResource::Unknown for {}: {:?}", &uri, execute.err()); }
                                                                                }
                                                                            }
                                                                            // TODO: why is this clone required?
                                                                            let cp_clone = canonical_path.clone();
                                                                            match extract_path_info(std::path::Path::new(&cp_clone), std::path::Path::new(&uri)) {
                                                                                Some((
                                                                                    file_path_abs,
                                                                                    file_path_rel_parent,
                                                                                    file_path_rel,
                                                                                    file_basename,
                                                                                    file_extn,
                                                                                )) => {
                                                                                    let ur_walk_session_path_fs_entry_id = ulid::Ulid::new().to_string();
                                                                                    match ur_fs_entry_stmt.execute(params![
                                                                                            ur_walk_session_path_fs_entry_id, walk_session_id, walk_path_id, uniform_resource_id,
                                                                                            file_path_abs.into_os_string().into_string().unwrap(), 
                                                                                            file_path_rel_parent.into_os_string().into_string().unwrap(), 
                                                                                            file_path_rel.into_os_string().into_string().unwrap(), file_basename, 
                                                                                            if let Some(file_extn) = file_extn { file_extn } else { String::from("") }
                                                                                            ]) {
                                                                                        Ok(_) => {},
                                                                                        Err(err) => { eprintln!("Error inserting UR walk session path file system entry for {}: {}", &uri, err); }
                                                                                    }
                                                                                },
                                                                                None => { eprintln!("Error extracting path info for {}", cp_clone); }
                                                                            }
                                                                        },
                                                                        Err(e) => { eprintln!("Error processing a resource: {}", e); },
                                                                    }
                                                                }
                                                            }
                                                            Err(err) => { print!("Error preparing walker: {err}");}
                                                        }     
                                                    }
                                                    Err(err) => { println!("ur_walk_session_path Error {}", err); }
                                            };
                                        }
                                        Err(err) => { print!("Error canonicalizing path {root_path}: {err}");}
                                        }
                                    }

                                    let _ = tx.execute("UPDATE ur_walk_session SET walk_finished_at = CURRENT_TIMESTAMP WHERE ur_walk_session_id = ?", params![walk_session_id]);
                                }
                                Err(err) => {
                                    println!("ur_walk_session Error {}", err);
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
