use std::path::PathBuf;

use clap::arg;

pub fn setup_cli() -> clap::Command {
    clap::Command::new("example")
        .about("A CLI tool for converting image files to DDS using Nvidia Texture CLI")
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
}
