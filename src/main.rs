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
    /// Database maintenance utilities
    DbMaint {
        /// target SQLite database
        #[arg(
            short,
            long,
            default_value = "./resource-surveillance.sqlite.db",
            default_missing_value = "always"
        )]
        db_fs_path: Option<String>,

        /// list notebooks and cells
        #[arg(long, short)]
        ls_notebooks: bool,
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

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.device_name.as_deref() {
        println!("Device: {name}");
    }

    if let Some(config_path) = cli.config.as_deref() {
        println!("config: {}", config_path.display());
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match cli.debug {
        0 => {}
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    match &cli.command {
        Some(Commands::DbMaint {
            db_fs_path,
            ls_notebooks,
        }) => {
            if let Some(db_fs_path) = db_fs_path.as_deref() {
                if let Ok(conn) =
                    Connection::open_with_flags(db_fs_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
                {
                    if *ls_notebooks {
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
                    }
                } else {
                    println!(
                        "DB Maintenance system could not open or create: {}",
                        db_fs_path
                    );
                };
            }
        }
        Some(Commands::FsWalk {
            root_path,
            ignore_entry,
            surveil_content,
            surveil_db_fs_path,
            compute_digests,
        }) => {
            if let Some(db_fs_path) = surveil_db_fs_path.as_deref() {
                println!("Surveillance DB URL: {db_fs_path}");

                if let Ok(conn) = Connection::open(db_fs_path) {
                    if let Ok(mut ctx) = RusqliteContext::new(&conn) {
                        match ctx.execute_migrations() {
                            Ok(_) => {}
                            Err(err) => {
                                println!("execute_migrations Error {}", err);
                            }
                        };
                        let _ = ctx.upserted_device(&DEVICE);
                    }
                } else {
                    println!("RusqliteContext Could not open or create: {}", db_fs_path);
                };
            }

            println!("Root paths: {}", root_path.join(", "));
            println!(
                "Ignore entries reg exes: {}",
                ignore_entry
                    .iter()
                    .map(|r| r.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ")
            );

            println!(
                "Compute digests reg exes: {}",
                compute_digests
                    .iter()
                    .map(|r| r.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ")
            );

            println!(
                "Content surveillance entries reg exes: {}",
                surveil_content
                    .iter()
                    .map(|r| r.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ")
            );

            let walker = FileSysResourcesWalker::new(root_path, ignore_entry, surveil_content);
            match walker {
                Ok(walker) => {
                    let _ = walker.walk_resources(|resource: UniformResource<ContentResource>| {
                        match resource {
                            UniformResource::Html(html) => {
                                println!("HTML: {:?} {:?}", html.resource.uri, html.resource.nature)
                            }
                            UniformResource::Json(json) => {
                                println!("JSON: {:?} {:?}", json.resource.uri, json.resource.nature)
                            }
                            UniformResource::Image(img) => {
                                println!("Image: {:?} {:?}", img.resource.uri, img.resource.nature)
                            }
                            UniformResource::Markdown(md) => {
                                println!("Markdown: {:?} {:?}", md.resource.uri, md.resource.nature)
                            }
                            UniformResource::Unknown(unknown) => {
                                println!("Unknown: {:?} {:?}", unknown.uri, unknown.nature)
                            }
                        }
                    });
                }
                Err(_) => {
                    print!("Error preparing walker")
                }
            }
        }
        None => {}
    }
}
