use anyhow::{Context, Ok};
use clap::Parser;

#[macro_use]
extern crate lazy_static;

mod device;
lazy_static! {
    static ref DEVICE: device::Device = device::Device::new(None);
}

#[macro_use]
mod helpers;

mod cmd;
mod format;
mod frontmatter;
mod fsresource;
mod persist;
mod resource;

fn main() -> anyhow::Result<()> {
    let cli = cmd::Cli::parse();

    if cli.help_markdown {
        clap_markdown::print_help_markdown::<cmd::Cli>();
        return Ok(());
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

    cli.command.execute(&cli).with_context(|| "main")?;
    Ok(())
}
