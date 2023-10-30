use std::path::PathBuf;

use clap::{Parser, Subcommand};
use regex::Regex;
// TODO: use regex::RegexSet;

#[macro_use]
extern crate lazy_static;

mod device;
lazy_static! {
    static ref DEVICE: device::Device = device::Device::new();
}

mod fsresource;
mod resource;
mod uniform;

use fsresource::*;
use resource::*;
use uniform::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
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

        /// reg-exes to use to load frontmatter for entry in addition to content
        #[arg(long, default_value = "\\.(md|mdx)$", default_missing_value = "always")]
        surveil_frontmatter: Vec<Regex>,
    },
}

fn main() {
    let cli = Cli::parse();

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
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::FsWalk {
            root_path,
            ignore_entry,
            surveil_content,
            surveil_frontmatter,
            compute_digests,
        }) => {
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

            println!(
                "Content frontmatter surveillance entries reg exes: {}",
                surveil_frontmatter
                    .iter()
                    .map(|r| r.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ")
            );

            let walker = FileSysResourcesWalker::new(root_path, ignore_entry, surveil_content);
            match walker {
                Ok(walker) => {
                    let _ =
                        walker.walk_resources(|resource: UniformResource<Resource<Vec<u8>>>| {
                            match resource {
                                UniformResource::HTML(html) => {
                                    println!(
                                        "HTML: {:?} {:?}",
                                        html.resource.uri, html.resource.nature
                                    )
                                }
                                UniformResource::JSON(json) => {
                                    println!(
                                        "JSON: {:?} {:?}",
                                        json.resource.uri, json.resource.nature
                                    )
                                }
                                UniformResource::Image(img) => {
                                    println!(
                                        "Image: {:?} {:?}",
                                        img.resource.uri, img.resource.nature
                                    )
                                }
                                UniformResource::Markdown(md) => {
                                    println!(
                                        "Markdown: {:?} {:?}",
                                        md.resource.uri, md.resource.nature
                                    )
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
