use std::path::PathBuf;

use clap::{arg, crate_description, crate_name, crate_version};

pub fn setup_cli() -> clap::Command {
    clap::Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .subcommand(clap::Command::new("install").about("Installs the application"))
        .subcommand(
            clap::Command::new("convert")
                .about("Converts one or more image files to .dds format")
                .arg_required_else_help(true)
                .arg(
                    arg!(<PATH> ... "Paths to image files to convert")
                        .value_parser(clap::value_parser!(PathBuf)),
                ),
        )
        .subcommand(clap::Command::new("uninstall").about("Uninstalls the application"))
}
