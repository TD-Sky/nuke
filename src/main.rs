mod cli;
mod error;
mod utils;

use std::io;
use std::process::Command;

use clap::Parser;
use indoc::formatdoc;
use utils::fs::virtually_exists;
use which::which;

use cli::Cli;
use error::Error;

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    if !virtually_exists("make.nu").map_err(Error::makefile)? {
        return Err(Error::makefile(io::Error::from(io::ErrorKind::NotFound)));
    }

    let plugin = which("nu_plugin_nuke").map_err(Error::plugin)?;

    let status = Command::new("nu")
        .args([
            "-c",
            &formatdoc! {"
                source make.nu
                {}",
                cli.nuke_schedule()
            },
            &format!("--plugins=[{}]", plugin.display()),
        ])
        .status()
        .map_err(Error::command)?;

    if !status.success() {
        return Err(Error::Nuke);
    }

    Ok(())
}
